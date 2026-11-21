local n = tonumber("3")
local out = {}
if n + 0 > 0 then
	table.insert(out, 0)
	if n + 1 > 1 then
		table.insert(out, 1)
		if n + 2 > 2 then
			table.insert(out, 2)
			if n + 3 > 3 then
				table.insert(out, 3)
				if n + 4 > 4 then
					table.insert(out, 4)
					if n + 5 > 0 then
						table.insert(out, 5)
						if n + 6 > 1 then
							table.insert(out, 6)
							if n + 7 > 2 then
								table.insert(out, 7)
								if n + 8 > 3 then
									table.insert(out, 8)
									if n + 9 > 4 then
										table.insert(out, 9)
										if n + 10 > 0 then
											table.insert(out, 10)
											if n + 11 > 1 then
												table.insert(out, 11)
												if n + 12 > 2 then
													table.insert(out, 12)
													if n + 13 > 3 then
														table.insert(out, 13)
														if n + 14 > 4 then
															table.insert(out, 14)
															if n + 15 > 0 then
																table.insert(out, 15)
																if n + 16 > 1 then
																	table.insert(out, 16)
																	if n + 17 > 2 then
																		table.insert(out, 17)
																		if n + 18 > 3 then
																			table.insert(out, 18)
																			if n + 19 > 4 then
																				table.insert(out, 19)
																				if n + 20 > 0 then
																					table.insert(out, 20)
																					if n + 21 > 1 then
																						table.insert(out, 21)
																						if n + 22 > 2 then
																							table.insert(out, 22)
																							if n + 23 > 3 then
																								table.insert(out, 23)
																								if n + 24 > 4 then
																									table.insert(out, 24)
																									if n + 25 > 0 then
																										table.insert(out, 25)
																										if n + 26 > 1 then
																											table.insert(out, 26)
																											if n + 27 > 2 then
																												table.insert(out, 27)
																												if n + 28 > 3 then
																													table.insert(out, 28)
																													if n + 29 > 4 then
																														table.insert(out, 29)
																														if n + 30 > 0 then
																															table.insert(out, 30)
																															if n + 31 > 1 then
																																table.insert(out, 31)
																																if n + 32 > 2 then
																																	table.insert(out, 32)
																																	if n + 33 > 3 then
																																		table.insert(out, 33)
																																		if n + 34 > 4 then
																																			table.insert(out, 34)
																																			if n + 35 > 0 then
																																				table.insert(out, 35)
																																				if n + 36 > 1 then
																																					table.insert(out, 36)
																																					if n + 37 > 2 then
																																						table.insert(out, 37)
																																						if n + 38 > 3 then
																																							table.insert(out, 38)
																																							if n + 39 > 4 then
																																								table.insert(out, 39)
																																							else
																																								table.insert(out, -39)
																																							end
																																						else
																																							table.insert(out, -38)
																																						end
																																					else
																																						table.insert(out, -37)
																																					end
																																				else
																																					table.insert(out, -36)
																																				end
																																			else
																																				table.insert(out, -35)
																																			end
																																		else
																																			table.insert(out, -34)
																																		end
																																	else
																																		table.insert(out, -33)
																																	end
																																else
																																	table.insert(out, -32)
																																end
																															else
																																table.insert(out, -31)
																															end
																														else
																															table.insert(out, -30)
																														end
																													else
																														table.insert(out, -29)
																													end
																												else
																													table.insert(out, -28)
																												end
																											else
																												table.insert(out, -27)
																											end
																										else
																											table.insert(out, -26)
																										end
																									else
																										table.insert(out, -25)
																									end
																								else
																									table.insert(out, -24)
																								end
																							else
																								table.insert(out, -23)
																							end
																						else
																							table.insert(out, -22)
																						end
																					else
																						table.insert(out, -21)
																					end
																				else
																					table.insert(out, -20)
																				end
																			else
																				table.insert(out, -19)
																			end
																		else
																			table.insert(out, -18)
																		end
																	else
																		table.insert(out, -17)
																	end
																else
																	table.insert(out, -16)
																end
															else
																table.insert(out, -15)
															end
														else
															table.insert(out, -14)
														end
													else
														table.insert(out, -13)
													end
												else
													table.insert(out, -12)
												end
											else
												table.insert(out, -11)
											end
										else
											table.insert(out, -10)
										end
									else
										table.insert(out, -9)
									end
								else
									table.insert(out, -8)
								end
							else
								table.insert(out, -7)
							end
						else
							table.insert(out, -6)
						end
					else
						table.insert(out, -5)
					end
				else
					table.insert(out, -4)
				end
			else
				table.insert(out, -3)
			end
		else
			table.insert(out, -2)
		end
	else
		table.insert(out, -1)
	end
else
	table.insert(out, -0)
end
print(#out, out[1], out[#out])
local acc = 0
local v1 = (n + 0) % 2
acc = if v1 == 0 then acc + 0 elseif n > 0 then acc - 1 else acc * 2
local v2 = (n + 1) % 3
acc = if v2 == 0 then acc + 1 elseif n > 1 then acc - 1 else acc * 2
local v3 = (n + 2) % 4
acc = if v3 == 0 then acc + 2 elseif n > 2 then acc - 1 else acc * 2
local v4 = (n + 3) % 5
acc = if v4 == 0 then acc + 3 elseif n > 3 then acc - 1 else acc * 2
local v5 = (n + 4) % 6
acc = if v5 == 0 then acc + 4 elseif n > 4 then acc - 1 else acc * 2
local v6 = (n + 5) % 7
acc = if v6 == 0 then acc + 5 elseif n > 5 then acc - 1 else acc * 2
local v7 = (n + 6) % 8
acc = if v7 == 0 then acc + 6 elseif n > 6 then acc - 1 else acc * 2
local v8 = (n + 7) % 2
acc = if v8 == 0 then acc + 7 elseif n > 7 then acc - 1 else acc * 2
local v9 = (n + 8) % 3
acc = if v9 == 0 then acc + 8 elseif n > 8 then acc - 1 else acc * 2
local v10 = (n + 9) % 4
acc = if v10 == 0 then acc + 9 elseif n > 9 then acc - 1 else acc * 2
local v11 = (n + 10) % 5
acc = if v11 == 0 then acc + 10 elseif n > 10 then acc - 1 else acc * 2
local v12 = (n + 11) % 6
acc = if v12 == 0 then acc + 11 elseif n > 11 then acc - 1 else acc * 2
local v13 = (n + 12) % 7
acc = if v13 == 0 then acc + 12 elseif n > 12 then acc - 1 else acc * 2
local v14 = (n + 13) % 8
acc = if v14 == 0 then acc + 13 elseif n > 0 then acc - 1 else acc * 2
local v15 = (n + 14) % 2
acc = if v15 == 0 then acc + 14 elseif n > 1 then acc - 1 else acc * 2
local v16 = (n + 15) % 3
acc = if v16 == 0 then acc + 15 elseif n > 2 then acc - 1 else acc * 2
local v17 = (n + 16) % 4
acc = if v17 == 0 then acc + 16 elseif n > 3 then acc - 1 else acc * 2
local v18 = (n + 17) % 5
acc = if v18 == 0 then acc + 17 elseif n > 4 then acc - 1 else acc * 2
local v19 = (n + 18) % 6
acc = if v19 == 0 then acc + 18 elseif n > 5 then acc - 1 else acc * 2
local v20 = (n + 19) % 7
acc = if v20 == 0 then acc + 19 elseif n > 6 then acc - 1 else acc * 2
local v21 = (n + 20) % 8
acc = if v21 == 0 then acc + 20 elseif n > 7 then acc - 1 else acc * 2
local v22 = (n + 21) % 2
acc = if v22 == 0 then acc + 21 elseif n > 8 then acc - 1 else acc * 2
local v23 = (n + 22) % 3
acc = if v23 == 0 then acc + 22 elseif n > 9 then acc - 1 else acc * 2
local v24 = (n + 23) % 4
acc = if v24 == 0 then acc + 23 elseif n > 10 then acc - 1 else acc * 2
local v25 = (n + 24) % 5
acc = if v25 == 0 then acc + 24 elseif n > 11 then acc - 1 else acc * 2
local v26 = (n + 25) % 6
acc = if v26 == 0 then acc + 25 elseif n > 12 then acc - 1 else acc * 2
local v27 = (n + 26) % 7
acc = if v27 == 0 then acc + 26 elseif n > 0 then acc - 1 else acc * 2
local v28 = (n + 27) % 8
acc = if v28 == 0 then acc + 27 elseif n > 1 then acc - 1 else acc * 2
local v29 = (n + 28) % 2
acc = if v29 == 0 then acc + 28 elseif n > 2 then acc - 1 else acc * 2
local v30 = (n + 29) % 3
acc = if v30 == 0 then acc + 29 elseif n > 3 then acc - 1 else acc * 2
local v31 = (n + 30) % 4
acc = if v31 == 0 then acc + 30 elseif n > 4 then acc - 1 else acc * 2
local v32 = (n + 31) % 5
acc = if v32 == 0 then acc + 31 elseif n > 5 then acc - 1 else acc * 2
local v33 = (n + 32) % 6
acc = if v33 == 0 then acc + 32 elseif n > 6 then acc - 1 else acc * 2
local v34 = (n + 33) % 7
acc = if v34 == 0 then acc + 33 elseif n > 7 then acc - 1 else acc * 2
local v35 = (n + 34) % 8
acc = if v35 == 0 then acc + 34 elseif n > 8 then acc - 1 else acc * 2
local v36 = (n + 35) % 2
acc = if v36 == 0 then acc + 35 elseif n > 9 then acc - 1 else acc * 2
local v37 = (n + 36) % 3
acc = if v37 == 0 then acc + 36 elseif n > 10 then acc - 1 else acc * 2
local v38 = (n + 37) % 4
acc = if v38 == 0 then acc + 37 elseif n > 11 then acc - 1 else acc * 2
local v39 = (n + 38) % 5
acc = if v39 == 0 then acc + 38 elseif n > 12 then acc - 1 else acc * 2
local v40 = (n + 39) % 6
acc = if v40 == 0 then acc + 39 elseif n > 0 then acc - 1 else acc * 2
local v41 = (n + 40) % 7
acc = if v41 == 0 then acc + 40 elseif n > 1 then acc - 1 else acc * 2
local v42 = (n + 41) % 8
acc = if v42 == 0 then acc + 41 elseif n > 2 then acc - 1 else acc * 2
local v43 = (n + 42) % 2
acc = if v43 == 0 then acc + 42 elseif n > 3 then acc - 1 else acc * 2
local v44 = (n + 43) % 3
acc = if v44 == 0 then acc + 43 elseif n > 4 then acc - 1 else acc * 2
local v45 = (n + 44) % 4
acc = if v45 == 0 then acc + 44 elseif n > 5 then acc - 1 else acc * 2
local v46 = (n + 45) % 5
acc = if v46 == 0 then acc + 45 elseif n > 6 then acc - 1 else acc * 2
local v47 = (n + 46) % 6
acc = if v47 == 0 then acc + 46 elseif n > 7 then acc - 1 else acc * 2
local v48 = (n + 47) % 7
acc = if v48 == 0 then acc + 47 elseif n > 8 then acc - 1 else acc * 2
local v49 = (n + 48) % 8
acc = if v49 == 0 then acc + 48 elseif n > 9 then acc - 1 else acc * 2
local v50 = (n + 49) % 2
acc = if v50 == 0 then acc + 49 elseif n > 10 then acc - 1 else acc * 2
local v51 = (n + 50) % 3
acc = if v51 == 0 then acc + 50 elseif n > 11 then acc - 1 else acc * 2
local v52 = (n + 51) % 4
acc = if v52 == 0 then acc + 51 elseif n > 12 then acc - 1 else acc * 2
local v53 = (n + 52) % 5
acc = if v53 == 0 then acc + 52 elseif n > 0 then acc - 1 else acc * 2
local v54 = (n + 53) % 6
acc = if v54 == 0 then acc + 53 elseif n > 1 then acc - 1 else acc * 2
local v55 = (n + 54) % 7
acc = if v55 == 0 then acc + 54 elseif n > 2 then acc - 1 else acc * 2
local v56 = (n + 55) % 8
acc = if v56 == 0 then acc + 55 elseif n > 3 then acc - 1 else acc * 2
local v57 = (n + 56) % 2
acc = if v57 == 0 then acc + 56 elseif n > 4 then acc - 1 else acc * 2
local v58 = (n + 57) % 3
acc = if v58 == 0 then acc + 57 elseif n > 5 then acc - 1 else acc * 2
local v59 = (n + 58) % 4
acc = if v59 == 0 then acc + 58 elseif n > 6 then acc - 1 else acc * 2
local v60 = (n + 59) % 5
acc = if v60 == 0 then acc + 59 elseif n > 7 then acc - 1 else acc * 2
print(acc)
local hits = 0
hits = if n == 0 then hits + 1 else hits
hits = if n == 1 then hits + 1 else hits
hits = if n == 2 then hits + 1 else hits
hits = if n == 3 then hits + 1 else hits
hits = if n == 0 then hits + 1 else hits
hits = if n == 1 then hits + 1 else hits
hits = if n == 2 then hits + 1 else hits
hits = if n == 3 then hits + 1 else hits
hits = if n == 0 then hits + 1 else hits
hits = if n == 1 then hits + 1 else hits
hits = if n == 2 then hits + 1 else hits
hits = if n == 3 then hits + 1 else hits
hits = if n == 0 then hits + 1 else hits
hits = if n == 1 then hits + 1 else hits
hits = if n == 2 then hits + 1 else hits
hits = if n == 3 then hits + 1 else hits
hits = if n == 0 then hits + 1 else hits
hits = if n == 1 then hits + 1 else hits
hits = if n == 2 then hits + 1 else hits
hits = if n == 3 then hits + 1 else hits
hits = if n == 0 then hits + 1 else hits
hits = if n == 1 then hits + 1 else hits
hits = if n == 2 then hits + 1 else hits
hits = if n == 3 then hits + 1 else hits
hits = if n == 0 then hits + 1 else hits
hits = if n == 1 then hits + 1 else hits
hits = if n == 2 then hits + 1 else hits
hits = if n == 3 then hits + 1 else hits
hits = if n == 0 then hits + 1 else hits
hits = if n == 1 then hits + 1 else hits
hits = if n == 2 then hits + 1 else hits
hits = if n == 3 then hits + 1 else hits
hits = if n == 0 then hits + 1 else hits
hits = if n == 1 then hits + 1 else hits
hits = if n == 2 then hits + 1 else hits
hits = if n == 3 then hits + 1 else hits
hits = if n == 0 then hits + 1 else hits
hits = if n == 1 then hits + 1 else hits
hits = if n == 2 then hits + 1 else hits
hits = if n == 3 then hits + 1 else hits
hits = if n == 0 then hits + 1 else hits
hits = if n == 1 then hits + 1 else hits
hits = if n == 2 then hits + 1 else hits
hits = if n == 3 then hits + 1 else hits
hits = if n == 0 then hits + 1 else hits
hits = if n == 1 then hits + 1 else hits
hits = if n == 2 then hits + 1 else hits
hits = if n == 3 then hits + 1 else hits
hits = if n == 0 then hits + 1 else hits
hits = if n == 1 then hits + 1 else hits
hits = if n == 2 then hits + 1 else hits
hits = if n == 3 then hits + 1 else hits
hits = if n == 0 then hits + 1 else hits
hits = if n == 1 then hits + 1 else hits
hits = if n == 2 then hits + 1 else hits
hits = if n == 3 then hits + 1 else hits
hits = if n == 0 then hits + 1 else hits
hits = if n == 1 then hits + 1 else hits
hits = if n == 2 then hits + 1 else hits
hits = if n == 3 then hits + 1 else hits
print(hits)
