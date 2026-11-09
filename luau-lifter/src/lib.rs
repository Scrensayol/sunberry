mod deserializer;
mod instruction;
mod lifter;
mod op_code;

use ast::{
    limit_locals::limit_locals, local_declarations::LocalDeclarer, method_sugar::method_sugar,
    redundant_continue::remove_redundant_continues,
    name_locals::name_module_locals,
    replace_locals::replace_locals,
    Traverse,
};

use by_address::ByAddress;
use cfg::{
    function::Function,
    ssa::{
        self,
        structuring::{structure_conditionals, structure_jumps},
    },
};
use indexmap::IndexMap;
use rayon::prelude::*;

use lifter::Lifter;

//use cfg_ir::{dot, function::Function, ssa};
use clap::Parser;
use parking_lot::Mutex;
use ast::dominators::simple_fast;

use rustc_hash::{FxHashMap, FxHashSet};
use triomphe::Arc;

use deserializer::bytecode::Bytecode;

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    paths: Vec<String>,
    /// Number of threads to use (0 = automatic)
    #[clap(short, long, default_value_t = 0)]
    threads: usize,
    /// op = op * key % 256
    /// For Roblox client bytecode, use 203
    #[clap(short, long, default_value_t = 1)]
    key: u8,
    #[clap(short, long)]
    recursive: bool,
    #[clap(short, long)]
    verbose: bool,
}

pub fn decompile_bytecode(bytecode: &[u8], encode_key: u8) -> String {
    decompile_script(bytecode, encode_key, None)
}

// `script_name` is the name of the script the bytecode came from, if known. A module that
// returns a local names that local after it.
pub fn decompile_script(bytecode: &[u8], encode_key: u8, script_name: Option<&str>) -> String {
    let chunk = deserializer::deserialize(bytecode, encode_key).unwrap();
    match chunk {
        Bytecode::Error(msg) => msg,
        Bytecode::Chunk(chunk) => {
            let main_ast = Arc::<Mutex<ast::Function>>::default();
            let mut lifted = Vec::new();
            let mut wave: Vec<(Arc<Mutex<ast::Function>>, usize)> = Vec::new();

            // lift main on its own so its arc stays unique, try_unwrap needs that later
            let main_lifted = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                ast::seed_local_ids(lift_seed(chunk.main));
                Lifter::lift(&chunk.functions, &chunk.string_table, chunk.main)
            })) {
                Ok((function, upvalues, child_functions)) => {
                    wave.extend(child_functions.into_iter().map(|(a, f)| (a.0, f)));
                    Some((function, upvalues))
                }
                Err(e) => {
                    let msg = e.downcast_ref::<String>().map(String::as_str)
                        .or_else(|| e.downcast_ref::<&str>().copied())
                        .unwrap_or("unknown panic");
                    main_ast.lock().body.push(
                        ast::Comment::new(format!("failed to lift main: {}", msg)).into(),
                    );
                    None
                }
            };

            // lifting is independent per function too, so lift in waves: every function
            // found by one wave gets lifted in parallel in the next
            while !wave.is_empty() {
                let results = std::mem::take(&mut wave)
                    .into_par_iter()
                    .map(|(ast_func, func_id)| {
                        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            ast::seed_local_ids(lift_seed(func_id));
                            Lifter::lift(&chunk.functions, &chunk.string_table, func_id)
                        }));
                        (ast_func, func_id, result)
                    })
                    .collect::<Vec<_>>();
                for (ast_func, func_id, result) in results {
                    match result {
                        Ok((function, upvalues, child_functions)) => {
                            lifted.push((ast_func, function, upvalues));
                            wave.extend(child_functions.into_iter().map(|(a, f)| (a.0, f)));
                        }
                        Err(e) => {
                            let msg = e.downcast_ref::<String>().map(String::as_str)
                                .or_else(|| e.downcast_ref::<&str>().copied())
                                .unwrap_or("unknown panic");
                            ast_func.lock().body.push(
                                ast::Comment::new(format!(
                                    "failed to lift function {}: {}", func_id, msg
                                )).into(),
                            );
                        }
                    }
                }
            }

            use std::{backtrace::Backtrace, cell::RefCell, panic};

            thread_local! {
                static BACKTRACE: RefCell<Option<Backtrace>> = const { RefCell::new(None) };
            }

            // the hook is process wide, so set it once around the parallel section
            let prev_hook = panic::take_hook();
            panic::set_hook(Box::new(|_| {
                let trace = Backtrace::capture();
                BACKTRACE.with(move |b| b.borrow_mut().replace(trace));
            }));

            // functions decompile independently: a parent only reaches a child's ast
            // through fold_classes, which takes the lock and is idempotent. that holds
            // for main as well, so it runs alongside the others
            let decompile_children = move || {
                lifted
                    .into_par_iter()
                    .map(|(ast_function, function, upvalues_in)| {
                        let function_id = function.id;
                        // one clone so the panic branch can still write to the ast, the closure takes the other
                        let mut args = std::panic::AssertUnwindSafe(Some((
                            ast_function.clone(),
                            function,
                            upvalues_in,
                        )));

                        let result = panic::catch_unwind(move || {
                            let (ast_function, function, upvalues_in) = args.take().unwrap();
                            decompile_function(ast_function, function, upvalues_in)
                        });

                        match result {
                            Ok(r) => r,
                            Err(e) => {
                                let msg = e.downcast_ref::<String>().map(String::as_str)
                                    .or_else(|| e.downcast_ref::<&str>().copied())
                                    .unwrap_or("unknown panic");
                                ast_function.lock().body.push(
                                    ast::Comment::new(format!(
                                        "failed to decompile function {}: {}", function_id, msg
                                    )).into(),
                                );
                                (ByAddress(ast_function), Vec::new())
                            }
                        }
                    })
                    .collect::<FxHashMap<_, _>>()
            };

            // main_ast is moved into the closure, so on panic theres nothing left to
            // write to and we have to make up a body instead
            let decompile_main = move || match main_lifted {
                Some((function, upvalues_in)) => {
                    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
                        decompile_function(main_ast, function, upvalues_in)
                    })) {
                        Ok((ba, _)) => Arc::try_unwrap(ba.0).unwrap().into_inner().body,
                        Err(e) => {
                            let msg = e.downcast_ref::<String>().map(String::as_str)
                                .or_else(|| e.downcast_ref::<&str>().copied())
                                .unwrap_or("unknown panic");
                            let mut block = ast::Block::default();
                            block.push(
                                ast::Comment::new(format!(
                                    "failed to decompile main: {}", msg
                                )).into(),
                            );
                            block
                        }
                    }
                }
                None => Arc::try_unwrap(main_ast).unwrap().into_inner().body,
            };

            let (mut upvalues, mut body) = rayon::join(decompile_children, decompile_main);
            panic::set_hook(prev_hook);
            link_upvalues(&mut body, &mut upvalues);
            method_sugar(&mut body);
            name_module_locals(&mut body, true, script_name);
            remove_redundant_continues(&mut body);
            limit_locals(&mut body);
            ast::formatter::Formatter::format_parallel(&body, Default::default())
        }
    }
}

// every function gets its own id range for locals made while lifting it and another for
// decompiling it, see ast::seed_local_ids
fn lift_seed(function_id: usize) -> u64 {
    (function_id as u64 * 2) << 32
}

fn decompile_seed(function_id: usize) -> u64 {
    (function_id as u64 * 2 + 1) << 32
}

fn decompile_function(
    ast_function: Arc<Mutex<ast::Function>>,
    mut function: Function,
    upvalues_in: Vec<ast::RcLocal>,
) -> (ByAddress<Arc<Mutex<ast::Function>>>, Vec<ast::RcLocal>) {
    ast::seed_local_ids(decompile_seed(function.id));
    let (local_count, local_groups, upvalue_in_groups, upvalue_passed_groups) =
        cfg::ssa::construct(&mut function, &upvalues_in);
    let upvalue_to_group = upvalue_in_groups
        .into_iter()
        .chain(
            upvalue_passed_groups
                .into_iter()
                .map(|m| (ast::RcLocal::default(), m)),
        )
        .flat_map(|(i, g)| g.into_iter().map(move |u| (u, i.clone())))
        .collect::<IndexMap<_, _>>();
    // TODO: do we even need this?
    let local_to_group = local_groups
        .into_iter()
        .enumerate()
        .flat_map(|(i, g)| g.into_iter().map(move |l| (l, i)))
        .collect::<FxHashMap<_, _>>();
    // TODO: REFACTOR: some way to write a macro that states
    // if cfg::ssa::inline results in change then structure_jumps, structure_compound_conditionals,
    // structure_for_loops and remove_unnecessary_params must run again.
    // if structure_compound_conditionals results in change then dominators and post dominators
    // must be recalculated.
    // etc.
    // the macro could also maybe generate an optimal ordering?
    let mut changed = true;
    while changed {
        changed = false;

        let dominators = simple_fast(function.graph(), function.entry().unwrap());
        changed |= structure_jumps(&mut function, &dominators);

        ssa::inline::inline(&mut function, &local_to_group, &upvalue_to_group);

        if structure_conditionals(&mut function)
        // || {
        //     let post_dominators = post_dominators(function.graph_mut());
        //     structure_for_loops(&mut function, &dominators, &post_dominators)
        // }
        // we can't structure method calls like this because of __namecall
        // || structure_method_calls(&mut function)
        {
            changed = true;
        }
        let mut local_map = FxHashMap::default();
        // TODO: loop until returns false?
        if ssa::construct::remove_unnecessary_params(&mut function, &mut local_map) {
            changed = true;
        }
        ssa::construct::apply_local_map(&mut function, local_map);
    }
    // cfg::dot::render_to(&function, &mut std::io::stdout()).unwrap();
    ssa::Destructor::new(
        &mut function,
        upvalue_to_group,
        upvalues_in.iter().cloned().collect(),
        local_count,
    )
    .destruct();

    let params = std::mem::take(&mut function.parameters);
    let is_variadic = function.is_variadic;
    let outer_locals = upvalues_in.iter().chain(params.iter()).cloned().collect();
    let block: Arc<Mutex<ast::Block>> =
        Arc::new(restructure::lift(function, FxHashSet::clone(&outer_locals)).into());
    ast::escaping_counters::split_escaping_counters(&mut block.lock());
    ast::fold_classes::fold_classes(&mut block.lock());
    LocalDeclarer::default().declare_locals(
        // TODO: why does block.clone() not work?
        Arc::clone(&block),
        &outer_locals,
    );

    {
        let mut ast_function = ast_function.lock();
        ast_function.body = Arc::try_unwrap(block).unwrap().into_inner();
        ast_function.parameters = params;
        ast_function.is_variadic = is_variadic;
    }
    (ByAddress(ast_function), upvalues_in)
}

fn link_upvalues(
    body: &mut ast::Block,
    upvalues: &mut FxHashMap<ByAddress<Arc<Mutex<ast::Function>>>, Vec<ast::RcLocal>>,
) {
    for stat in &mut body.0 {
        stat.traverse_rvalues(&mut |rvalue| {
            if let ast::RValue::Closure(closure) = rvalue {
                if let Some(old_upvalues) = upvalues.get(&closure.function) {
                    let mut function = closure.function.lock();
                    // TODO: inefficient, try constructing a map of all up -> new up first
                    // and then call replace_locals on main body
                    let mut local_map =
                        FxHashMap::with_capacity_and_hasher(old_upvalues.len(), Default::default());
                    for (old, new) in
                        old_upvalues
                            .iter()
                            .zip(closure.upvalues.iter().map(|u| match u {
                                ast::Upvalue::Copy(l) | ast::Upvalue::Ref(l) => l,
                            }))
                    {
                        // println!("{} -> {}", old, new);
                        local_map.insert(old.clone(), new.clone());
                    }
                    link_upvalues(&mut function.body, upvalues);
                    replace_locals(&mut function.body, &local_map);
                }
            }
        });
        match stat {
            ast::Statement::If(r#if) => {
                link_upvalues(&mut r#if.then_block.lock(), upvalues);
                link_upvalues(&mut r#if.else_block.lock(), upvalues);
            }
            ast::Statement::While(r#while) => {
                link_upvalues(&mut r#while.block.lock(), upvalues);
            }
            ast::Statement::Repeat(repeat) => {
                link_upvalues(&mut repeat.block.lock(), upvalues);
            }
            ast::Statement::NumericFor(numeric_for) => {
                link_upvalues(&mut numeric_for.block.lock(), upvalues);
            }
            ast::Statement::GenericFor(generic_for) => {
                link_upvalues(&mut generic_for.block.lock(), upvalues);
            }
            _ => {}
        }
    }
}
