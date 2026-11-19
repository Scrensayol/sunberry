use crate::generator::{Node, Program};

// Deletes whole statements (and their nested bodies) for as long as the program keeps
// failing the same way. Deletions that break the syntax or change how it fails are
// rejected by the predicate, so the generator never needs to know about the reducer.
pub fn reduce(program: &Program, still_fails: impl Fn(&Program) -> bool, budget: usize) -> (Program, usize) {
    let mut current = program.clone();
    let mut attempts = 0;
    let mut index = 0;
    loop {
        let paths = paths(&current);
        let Some(path) = paths.get(index) else { break };
        if attempts >= budget {
            break;
        }
        attempts += 1;
        let candidate = remove(&current, path);
        if still_fails(&candidate) {
            current = candidate;
        } else {
            index += 1;
        }
    }
    (current, attempts)
}

fn paths(program: &Program) -> Vec<Vec<usize>> {
    fn walk(nodes: &[Node], prefix: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
        for (i, node) in nodes.iter().enumerate() {
            prefix.push(i);
            out.push(prefix.clone());
            walk(&node.body, prefix, out);
            prefix.pop();
        }
    }
    let mut out = Vec::new();
    walk(&program.units, &mut Vec::new(), &mut out);
    out
}

fn remove(program: &Program, path: &[usize]) -> Program {
    let mut copy = program.clone();
    let mut nodes = &mut copy.units;
    for &i in &path[..path.len() - 1] {
        nodes = &mut nodes[i].body;
    }
    nodes.remove(path[path.len() - 1]);
    copy
}
