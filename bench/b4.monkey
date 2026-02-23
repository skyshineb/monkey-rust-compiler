let makeTransformer = fn(seed) {
  let addSeed = fn(x) { x + seed; };
  let mul = fn(x) { x * 3; };
  fn(v) {
    let a = addSeed(v);
    let b = mul(a);
    b + seed;
  };
};

let WORK = 15000;
let ROUNDS = 4;

let round = 0;
let checksum = 0;

while (round < ROUNDS) {
  let f = makeTransformer(round + 1);
  let i = 0;
  let local = 0;

  while (i < WORK) {
    let local = local + f(i);
    let i = i + 1;
  };

  let checksum = checksum + local;
  let round = round + 1;
};

puts("profile =", "closure-heavy");
puts("work =", WORK);
puts("rounds =", ROUNDS);
puts("checksum =", checksum);
