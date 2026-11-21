-- branch-heavy code (a friend's challenge): ifs nested 40 deep, 60 if/elseif/else in a row and
-- 60 conditions with a dead `false and` part. A decompiler that retries a nested `if` on a
-- failed attempt takes 2^depth steps here
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
if (n + 0) % 2 == 0 then
	acc += 0
elseif n > 0 then
	acc -= 1
else
	acc *= 2
end
if (n + 1) % 3 == 0 then
	acc += 1
elseif n > 1 then
	acc -= 1
else
	acc *= 2
end
if (n + 2) % 4 == 0 then
	acc += 2
elseif n > 2 then
	acc -= 1
else
	acc *= 2
end
if (n + 3) % 5 == 0 then
	acc += 3
elseif n > 3 then
	acc -= 1
else
	acc *= 2
end
if (n + 4) % 6 == 0 then
	acc += 4
elseif n > 4 then
	acc -= 1
else
	acc *= 2
end
if (n + 5) % 7 == 0 then
	acc += 5
elseif n > 5 then
	acc -= 1
else
	acc *= 2
end
if (n + 6) % 8 == 0 then
	acc += 6
elseif n > 6 then
	acc -= 1
else
	acc *= 2
end
if (n + 7) % 2 == 0 then
	acc += 7
elseif n > 7 then
	acc -= 1
else
	acc *= 2
end
if (n + 8) % 3 == 0 then
	acc += 8
elseif n > 8 then
	acc -= 1
else
	acc *= 2
end
if (n + 9) % 4 == 0 then
	acc += 9
elseif n > 9 then
	acc -= 1
else
	acc *= 2
end
if (n + 10) % 5 == 0 then
	acc += 10
elseif n > 10 then
	acc -= 1
else
	acc *= 2
end
if (n + 11) % 6 == 0 then
	acc += 11
elseif n > 11 then
	acc -= 1
else
	acc *= 2
end
if (n + 12) % 7 == 0 then
	acc += 12
elseif n > 12 then
	acc -= 1
else
	acc *= 2
end
if (n + 13) % 8 == 0 then
	acc += 13
elseif n > 0 then
	acc -= 1
else
	acc *= 2
end
if (n + 14) % 2 == 0 then
	acc += 14
elseif n > 1 then
	acc -= 1
else
	acc *= 2
end
if (n + 15) % 3 == 0 then
	acc += 15
elseif n > 2 then
	acc -= 1
else
	acc *= 2
end
if (n + 16) % 4 == 0 then
	acc += 16
elseif n > 3 then
	acc -= 1
else
	acc *= 2
end
if (n + 17) % 5 == 0 then
	acc += 17
elseif n > 4 then
	acc -= 1
else
	acc *= 2
end
if (n + 18) % 6 == 0 then
	acc += 18
elseif n > 5 then
	acc -= 1
else
	acc *= 2
end
if (n + 19) % 7 == 0 then
	acc += 19
elseif n > 6 then
	acc -= 1
else
	acc *= 2
end
if (n + 20) % 8 == 0 then
	acc += 20
elseif n > 7 then
	acc -= 1
else
	acc *= 2
end
if (n + 21) % 2 == 0 then
	acc += 21
elseif n > 8 then
	acc -= 1
else
	acc *= 2
end
if (n + 22) % 3 == 0 then
	acc += 22
elseif n > 9 then
	acc -= 1
else
	acc *= 2
end
if (n + 23) % 4 == 0 then
	acc += 23
elseif n > 10 then
	acc -= 1
else
	acc *= 2
end
if (n + 24) % 5 == 0 then
	acc += 24
elseif n > 11 then
	acc -= 1
else
	acc *= 2
end
if (n + 25) % 6 == 0 then
	acc += 25
elseif n > 12 then
	acc -= 1
else
	acc *= 2
end
if (n + 26) % 7 == 0 then
	acc += 26
elseif n > 0 then
	acc -= 1
else
	acc *= 2
end
if (n + 27) % 8 == 0 then
	acc += 27
elseif n > 1 then
	acc -= 1
else
	acc *= 2
end
if (n + 28) % 2 == 0 then
	acc += 28
elseif n > 2 then
	acc -= 1
else
	acc *= 2
end
if (n + 29) % 3 == 0 then
	acc += 29
elseif n > 3 then
	acc -= 1
else
	acc *= 2
end
if (n + 30) % 4 == 0 then
	acc += 30
elseif n > 4 then
	acc -= 1
else
	acc *= 2
end
if (n + 31) % 5 == 0 then
	acc += 31
elseif n > 5 then
	acc -= 1
else
	acc *= 2
end
if (n + 32) % 6 == 0 then
	acc += 32
elseif n > 6 then
	acc -= 1
else
	acc *= 2
end
if (n + 33) % 7 == 0 then
	acc += 33
elseif n > 7 then
	acc -= 1
else
	acc *= 2
end
if (n + 34) % 8 == 0 then
	acc += 34
elseif n > 8 then
	acc -= 1
else
	acc *= 2
end
if (n + 35) % 2 == 0 then
	acc += 35
elseif n > 9 then
	acc -= 1
else
	acc *= 2
end
if (n + 36) % 3 == 0 then
	acc += 36
elseif n > 10 then
	acc -= 1
else
	acc *= 2
end
if (n + 37) % 4 == 0 then
	acc += 37
elseif n > 11 then
	acc -= 1
else
	acc *= 2
end
if (n + 38) % 5 == 0 then
	acc += 38
elseif n > 12 then
	acc -= 1
else
	acc *= 2
end
if (n + 39) % 6 == 0 then
	acc += 39
elseif n > 0 then
	acc -= 1
else
	acc *= 2
end
if (n + 40) % 7 == 0 then
	acc += 40
elseif n > 1 then
	acc -= 1
else
	acc *= 2
end
if (n + 41) % 8 == 0 then
	acc += 41
elseif n > 2 then
	acc -= 1
else
	acc *= 2
end
if (n + 42) % 2 == 0 then
	acc += 42
elseif n > 3 then
	acc -= 1
else
	acc *= 2
end
if (n + 43) % 3 == 0 then
	acc += 43
elseif n > 4 then
	acc -= 1
else
	acc *= 2
end
if (n + 44) % 4 == 0 then
	acc += 44
elseif n > 5 then
	acc -= 1
else
	acc *= 2
end
if (n + 45) % 5 == 0 then
	acc += 45
elseif n > 6 then
	acc -= 1
else
	acc *= 2
end
if (n + 46) % 6 == 0 then
	acc += 46
elseif n > 7 then
	acc -= 1
else
	acc *= 2
end
if (n + 47) % 7 == 0 then
	acc += 47
elseif n > 8 then
	acc -= 1
else
	acc *= 2
end
if (n + 48) % 8 == 0 then
	acc += 48
elseif n > 9 then
	acc -= 1
else
	acc *= 2
end
if (n + 49) % 2 == 0 then
	acc += 49
elseif n > 10 then
	acc -= 1
else
	acc *= 2
end
if (n + 50) % 3 == 0 then
	acc += 50
elseif n > 11 then
	acc -= 1
else
	acc *= 2
end
if (n + 51) % 4 == 0 then
	acc += 51
elseif n > 12 then
	acc -= 1
else
	acc *= 2
end
if (n + 52) % 5 == 0 then
	acc += 52
elseif n > 0 then
	acc -= 1
else
	acc *= 2
end
if (n + 53) % 6 == 0 then
	acc += 53
elseif n > 1 then
	acc -= 1
else
	acc *= 2
end
if (n + 54) % 7 == 0 then
	acc += 54
elseif n > 2 then
	acc -= 1
else
	acc *= 2
end
if (n + 55) % 8 == 0 then
	acc += 55
elseif n > 3 then
	acc -= 1
else
	acc *= 2
end
if (n + 56) % 2 == 0 then
	acc += 56
elseif n > 4 then
	acc -= 1
else
	acc *= 2
end
if (n + 57) % 3 == 0 then
	acc += 57
elseif n > 5 then
	acc -= 1
else
	acc *= 2
end
if (n + 58) % 4 == 0 then
	acc += 58
elseif n > 6 then
	acc -= 1
else
	acc *= 2
end
if (n + 59) % 5 == 0 then
	acc += 59
elseif n > 7 then
	acc -= 1
else
	acc *= 2
end
print(acc)

local hits = 0
if n == 0 or false and n then
	hits += 1
end
if n == 1 or false and n then
	hits += 1
end
if n == 2 or false and n then
	hits += 1
end
if n == 3 or false and n then
	hits += 1
end
if n == 0 or false and n then
	hits += 1
end
if n == 1 or false and n then
	hits += 1
end
if n == 2 or false and n then
	hits += 1
end
if n == 3 or false and n then
	hits += 1
end
if n == 0 or false and n then
	hits += 1
end
if n == 1 or false and n then
	hits += 1
end
if n == 2 or false and n then
	hits += 1
end
if n == 3 or false and n then
	hits += 1
end
if n == 0 or false and n then
	hits += 1
end
if n == 1 or false and n then
	hits += 1
end
if n == 2 or false and n then
	hits += 1
end
if n == 3 or false and n then
	hits += 1
end
if n == 0 or false and n then
	hits += 1
end
if n == 1 or false and n then
	hits += 1
end
if n == 2 or false and n then
	hits += 1
end
if n == 3 or false and n then
	hits += 1
end
if n == 0 or false and n then
	hits += 1
end
if n == 1 or false and n then
	hits += 1
end
if n == 2 or false and n then
	hits += 1
end
if n == 3 or false and n then
	hits += 1
end
if n == 0 or false and n then
	hits += 1
end
if n == 1 or false and n then
	hits += 1
end
if n == 2 or false and n then
	hits += 1
end
if n == 3 or false and n then
	hits += 1
end
if n == 0 or false and n then
	hits += 1
end
if n == 1 or false and n then
	hits += 1
end
if n == 2 or false and n then
	hits += 1
end
if n == 3 or false and n then
	hits += 1
end
if n == 0 or false and n then
	hits += 1
end
if n == 1 or false and n then
	hits += 1
end
if n == 2 or false and n then
	hits += 1
end
if n == 3 or false and n then
	hits += 1
end
if n == 0 or false and n then
	hits += 1
end
if n == 1 or false and n then
	hits += 1
end
if n == 2 or false and n then
	hits += 1
end
if n == 3 or false and n then
	hits += 1
end
if n == 0 or false and n then
	hits += 1
end
if n == 1 or false and n then
	hits += 1
end
if n == 2 or false and n then
	hits += 1
end
if n == 3 or false and n then
	hits += 1
end
if n == 0 or false and n then
	hits += 1
end
if n == 1 or false and n then
	hits += 1
end
if n == 2 or false and n then
	hits += 1
end
if n == 3 or false and n then
	hits += 1
end
if n == 0 or false and n then
	hits += 1
end
if n == 1 or false and n then
	hits += 1
end
if n == 2 or false and n then
	hits += 1
end
if n == 3 or false and n then
	hits += 1
end
if n == 0 or false and n then
	hits += 1
end
if n == 1 or false and n then
	hits += 1
end
if n == 2 or false and n then
	hits += 1
end
if n == 3 or false and n then
	hits += 1
end
if n == 0 or false and n then
	hits += 1
end
if n == 1 or false and n then
	hits += 1
end
if n == 2 or false and n then
	hits += 1
end
if n == 3 or false and n then
	hits += 1
end
print(hits)
