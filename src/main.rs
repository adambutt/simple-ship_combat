use simple_ship_combat::*;

fn main() {
	let p1 = get_player_name(1);
	let p2 = get_player_name(2);
	let mut g = Game::new(&p1, &p2);

	print_hp(g.player1());
	print_hp(g.player2());

	loop {
		//play turn
		g.play_round(alloc_actions, alloc_targeting, alloc_countermeasures);

		print_hp(g.player1());
		print_hp(g.player2());

		match g.winner() {
			Winner::Draw => {
				println!("Draw:  Both player ships were destroyed.");
				break;
			},
			Winner::Player(p) => {
				println!("{} wone with {} remaining HP.", p.name(), p.hp());
				break;
			},
			Winner::None => continue,
		}
	}
}

fn print_hp(p: &Player) {
	println!("{} HP: {}", p.name(), p.hp());
}

fn alloc_actions(p: &Player, pa: &mut PoolAllocation){
	println!("{}: allocate your {} actions into pools for attack, targeting, and countermeasures (e.g. 3 2 1).", p.name(), pa.actions());
	
	loop {
		match read_values() {
			Err(e) => {
				println!("Invalid: {}.", e);
				continue;
			},
			Ok(v) => {
				if v.len() > 3 {
					println!("Invalid: There are only 3 pools you can allocate into.");
					continue;
				}
				pa.attack_pool = v.get(0).copied().unwrap_or(0);
				pa.targeting_pool = v.get(1).copied().unwrap_or(0);
				pa.countermeasure_pool = v.get(2).copied().unwrap_or(0);
			}
		}

		let total = pa.attack_pool + pa.targeting_pool + pa.countermeasure_pool;
		if total < pa.actions() {
			println!("Invalid: You really should allocate all your actions if you want to win.");
			continue;
		} else if total > pa.actions() {
			println!("Invalid: You don't have that many actions.");
			continue;
		}

		if !pa.is_valid() {
			println!("Invalid: Specified allocation is not allowed.");
			continue;
		}
		
		println!("Attack Pool: {}, Targeting Pool: {}, Countermeasure Pool: {}.", pa.attack_pool, pa.targeting_pool, pa.countermeasure_pool);
		return;
	}
}

fn alloc_targeting(p: &Player, ta: &mut TargetingAllocation) {
	println!("{}: allocate your targeting dice [{}] with your attack dice [{}] (e.g. 0 3 2).", p.name(), dice_list(ta.targeting_dice()), dice_list(ta.attack_dice()));
	
	loop {
		match read_values() {
			Err(e) => {
				println!("Invalid: {}.", e);
				continue;
			},
			Ok(v) => {
				if v.len() > ta.attack_dice().len() {
					println!("Invalid: You don't have that many attack dice to allocate to.");
					continue;
				}

				for (i, v) in v.iter().enumerate() {
					*ta.targeting_allocation_mut().get_mut(i).unwrap() = *v;
				}
			}
		}

		let count = ta.targeting_allocation().iter().filter(|&&v| v>0).count();
		if count < ta.attack_dice().len() && count < ta.targeting_dice().len() {
			println!("Invalid: You really should allocate all the dice you can.");
			continue;
		}

		if !ta.is_valid() {
			println!("Invalid: Specified allocation is not allowed.");
			continue;
		}
		
		println!("Attack & Targeting Allocations: [{}].", atk_tgt_list(ta.attack_dice(), ta.targeting_allocation()));
		return;
	}
}

fn alloc_countermeasures(p: &Player, ca: &mut CountermeasureAllocation) {
	println!("{}: allocate your countermeasure dice [{}] with your oponets attack/targeting dice [{}] (e.g. 0 3 2).", p.name(), dice_list(ca.countermeasure_dice()), atk_tgt_list(ca.opponent_attack_dice(), ca.opponent_targeting_allocation()));
	
	loop {
		match read_values() {
			Err(e) => {
				println!("Invalid: {}.", e);
				continue;
			},
			Ok(v) => {
				if v.len() > ca.opponent_attack_dice().len() {
					println!("Invalid: Your oponent doesn't have that many countermeasure dice to allocate to.");
					continue;
				}

				for (i, v) in v.iter().enumerate() {
					*ca.countermeasure_allocation_mut().get_mut(i).unwrap() = *v;
				}
			}
		}

		let count = ca.countermeasure_allocation().iter().filter(|&&v| v>0).count();
		if count < ca.opponent_attack_dice().len() && count < ca.countermeasure_dice().len() {
			println!("Invalid: You really should allocate all the dice you can.");
			continue;
		}

		if !ca.is_valid() {
			println!("Invalid: Specified allocation is not allowed.");
			continue;
		}
	
		println!("Countermeasure Allocations: [{}].", ctr_list(ca.opponent_attack_dice(), ca.opponent_targeting_allocation(), ca.countermeasure_allocation()));
		return;
	}
}

fn read_values<F>()->Result<Vec<F>, F::Err> 
	where F: std::str::FromStr {
	use std::io::Write;

	print!("»");
	std::io::stdout().flush().unwrap();

	let mut alloc_str = String::new();
	std::io::stdin()
		.read_line(&mut alloc_str)
		.expect("Failed to read input");
	
	alloc_str.split_whitespace()
		.map(|word| word.parse())
		.collect()
}

fn get_player_name(index: usize)-> String {
	let mut default = String::new();
	{
		use std::fmt::Write;
		write!(default, "Player {}", index).unwrap();
	}

	print!("Enter name for {0} [{0}]»", default);
	use std::io::Write;
	std::io::stdout().flush().unwrap();

	let mut name = String::new();
	std::io::stdin()
		.read_line(&mut name)
		.expect("Failed to read input");

	name = String::from(name.trim());
	if name.len() < 1 {
		name = default;
	}

	return name;
}

fn dice_list(d: &[u8]) -> String {
	use std::fmt::Write;

	let mut s = String::with_capacity(d.len()* 3);	
	for v in d {
		write!(&mut s, "{v}, ").unwrap();
	}
	
	s.pop();
	s.pop();
	
	return s;
}

fn atk_tgt_list(atk: &[u8], tgt: &[u8]) -> String {
	use std::fmt::Write;

	let mut s = String::with_capacity(atk.len()*5);
	for i in 0..atk.len() {
		write!(&mut s, "{}", atk[i]).unwrap();
		
		let t = tgt[i];
		if t > 0 {
			write!(&mut s, "+{t}").unwrap();
		}

		if i < atk.len()-1 {
			s.push_str(", ");
		}
	}

	return s;
}

fn ctr_list(atk: &[u8], tgt: &[u8], ctr: &[u8]) -> String {
	use std::fmt::Write;

	let mut s = String::with_capacity(atk.len()*6);
	for i in 0..atk.len() {
		write!(&mut s, "{}", atk[i]).unwrap();
		
		let t = tgt[i];
		if t > 0 {
			write!(&mut s, "+{t}").unwrap();
		}

		let c = ctr[i];
		if c > 0 {
			write!(&mut s, "|{c}").unwrap();
		}

		if i < atk.len()-1 {
			s.push_str(", ");
		}
	}

	return s;
}
