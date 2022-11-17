use rand::Rng;

const START_HP: u32 = 10;

pub enum Winner<'a> {
	 None,
	 Player(&'a Player<'a>),
	 Draw,
}

pub struct Game<'a> {
	p1: Player<'a>,
	p2: Player<'a>,
}

impl Game<'_> {
	pub fn new<'a>(player1: &'a str, player2: &'a str) -> Game<'a> {
		Game{
			p1: Player::new(player1),
			p2: Player::new(player2),
		}
	}

	pub fn player1(&self) -> &Player{
		return &self.p1;
	}

	pub fn player2(&self) -> &Player{
		return &self.p2;
	}

	pub fn winner(&self) -> Winner {		
		if self.p1.hp > 0 && self.p2.hp > 0{
			Winner::None
		} else if self.p1.hp > self.p2.hp {
			Winner::Player(&self.p1)
		} else if self.p1.hp < self.p2.hp {
			Winner::Player(&self.p2)
		} else {
			Winner::Draw
		}
	}

	pub fn play_round(
		&mut self,
		alloc_actions: fn(player: &Player, aa: &mut PoolAllocation),
		alloc_targeting: fn(player: &Player, ta: &mut TargetingAllocation),
		alloc_countermeasures: fn(player: &Player, ca: &mut CountermeasureAllocation),
	){
		let mut p1pool = allocate_pools(&self.p1, alloc_actions);
		let mut p2pool = allocate_pools(&self.p2, alloc_actions);

		//pre-size the countermeasure allocation vec.
		p1pool.countermeasure_allocation.resize(p2pool.attack_dice.len(), 0);
		p2pool.countermeasure_allocation.resize(p1pool.attack_dice.len(), 0);

		allocate_targeting(&self.p1, &mut p1pool, alloc_targeting);
		allocate_targeting(&self.p2, &mut p2pool, alloc_targeting);

		allocate_countermeasures(&self.p1, &mut p1pool, &p2pool, alloc_countermeasures);
		allocate_countermeasures(&self.p2, &mut p2pool, &p1pool, alloc_countermeasures);

		self.p1.hp = apply_damage(self.p1.hp, &p1pool, &p2pool);
		self.p2.hp = apply_damage(self.p2.hp, &p2pool, &p1pool);
	}
}

fn apply_damage(hp: u32, p_pool: &PlayerDice, o_pool: &PlayerDice)->u32{
	let mut dmg = 0u32;

	for (i, a) in o_pool.attack_dice.iter().copied().enumerate() {
		let t = o_pool.targeting_allocation[i];
		let c = p_pool.countermeasure_allocation[i];

		if a+t>c {
			dmg+=1;
		}
	}

	if dmg <= hp {
		hp-dmg
	}else {
		0
	}
}

pub struct Player<'a> {
	name: &'a str,
	hp: u32,
}

impl Player<'_> {
	fn new<'a>(name: &'a str) -> Player<'a> {
		Player { name: name, hp: START_HP }
	}

	pub fn name(&self) -> &str {
		self.name
	}

	pub fn hp(&self) -> u32 {
		self.hp
	}
}

fn allocate_pools(player: &Player, alloc_actions: fn(player: &Player, &mut PoolAllocation))->PlayerDice {
    //let players allocate pools
    let mut pa = PoolAllocation::new();
    alloc_actions(player, &mut pa);

    //verify results
    if !pa.is_valid() {
		panic!("pool allocation invalid");
	}

    //roll dice
    PlayerDice::new(&pa)
}

fn allocate_targeting(p: &Player, pd: &mut PlayerDice, alloc_targeting: fn(player: &Player, &mut TargetingAllocation)) {
    //let players allocate targeting dice
    let mut ta = TargetingAllocation{
		attack_dice: &pd.attack_dice,
		targeting_dice: &pd.targeting_dice,
		targeting_allocation: &mut pd.targeting_allocation,
	};
    alloc_targeting(p, &mut ta);

    //verify results
    if !ta.is_valid() {
		panic!("targeting allocation invalid");
	}
}

fn allocate_countermeasures(p: &Player, pd: &mut PlayerDice, o: & PlayerDice, alloc_countermeasures: fn(player: &Player, &mut CountermeasureAllocation)) {
    //let players allocate countermeasure dice
    let mut ca = CountermeasureAllocation{
		opponent_attack_dice: &o.attack_dice,
		opponent_targeting_allocation: &o.targeting_allocation,
		countermeasure_dice: &pd.countermeasure_dice,
		countermeasure_allocation: &mut pd.countermeasure_allocation,
	};
    alloc_countermeasures(p, &mut ca);

    //verify results
	if !ca.is_valid() {
		panic!("countermeasure allocation invalid");
	}
}

struct PlayerDice {
	attack_dice: Vec<u8>,
	targeting_dice: Vec<u8>,
	targeting_allocation: Vec<u8>,
	countermeasure_dice: Vec<u8>,
	countermeasure_allocation: Vec<u8>,
}

impl PlayerDice {
	fn new(a: &PoolAllocation) -> PlayerDice {
		let mut pd = PlayerDice {
			attack_dice: roll_dice(a.attack_pool),
			targeting_dice: roll_dice(a.targeting_pool),
			targeting_allocation: Vec::new(),
			countermeasure_dice: roll_dice(a.countermeasure_pool),
			countermeasure_allocation: Vec::new(),
		};

		pd.targeting_allocation.resize(a.attack_pool, 0);

		return pd;
	}
}

fn roll_dice(c: usize) -> Vec<u8> {
	let mut v = Vec::with_capacity(c);
	let mut rnd = rand::thread_rng();

	for _ in 0..c {
		v.push(rnd.gen_range(1..=6));
	}
	return v;
}

pub struct PoolAllocation {
	actions: usize,
	pub attack_pool: usize,
	pub targeting_pool: usize,
	pub countermeasure_pool: usize,
}

const DEFAULT_ACTIONS: usize = 6;

impl PoolAllocation {
	pub fn new() -> PoolAllocation{
		PoolAllocation {
			actions: DEFAULT_ACTIONS,
			attack_pool: 0,
			targeting_pool: 0,
			countermeasure_pool: 0,
		}
	}

	pub fn actions(&self) -> usize {self.actions}

	pub fn is_valid(&self) -> bool {
		self.actions >= self.attack_pool + self.targeting_pool + self.countermeasure_pool
	}
}

pub struct TargetingAllocation<'a> {
	attack_dice: &'a [u8],
	targeting_dice: &'a [u8],
	targeting_allocation: &'a mut [u8],
}

impl TargetingAllocation<'_> {
	pub fn attack_dice(&self) -> &[u8] { self.attack_dice}
	pub fn targeting_dice(&self) -> &[u8] { self.targeting_dice}
	pub fn targeting_allocation(&self) -> &[u8] { self.targeting_allocation}
	pub fn targeting_allocation_mut(&mut self) -> &mut[u8] { self.targeting_allocation}

	pub fn is_valid(&self) -> bool {
		let mut t_dice = Vec::from(self.targeting_dice);
		for t in self.targeting_allocation.iter().copied() {
			if t > 0 {
				match t_dice.iter().position(|d| *d == t) {
					None => return false,
					Some(i) => t_dice.swap_remove(i),
				};
			}
		}

		true
	}
}

pub struct CountermeasureAllocation<'a> {
	opponent_attack_dice: &'a [u8],
	opponent_targeting_allocation: &'a [u8],
	countermeasure_dice: &'a [u8],
	countermeasure_allocation: &'a mut [u8],
}

impl CountermeasureAllocation<'_> {
	pub fn opponent_attack_dice(&self) -> &[u8] { self.opponent_attack_dice}
	pub fn opponent_targeting_allocation(&self) -> &[u8] { self.opponent_targeting_allocation}
	pub fn countermeasure_dice(&self) -> &[u8] { self.countermeasure_dice}
	pub fn countermeasure_allocation(&self) -> &[u8] { self.countermeasure_allocation}
	pub fn countermeasure_allocation_mut(&mut self) -> &mut[u8] { self.countermeasure_allocation}

	pub fn is_valid(&self) -> bool {
		let mut c_dice = Vec::from(self.countermeasure_dice);
		for c in self.countermeasure_allocation.iter().copied() {
			if c > 0 {
				match c_dice.iter().position(|d| *d == c) {
					None => return false,
					Some(i) => c_dice.swap_remove(i),
				};
			}
		}

		true
	}
}
