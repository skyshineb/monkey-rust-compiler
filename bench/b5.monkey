let WORK = 20000;
let ROUNDS = 4;

let round = 0;
let checksum = 0;

while (round < ROUNDS) {
  let i = 0;
  while (i < WORK) {
    let h = {
      "a": i,
      "b": i + 1,
      "c": i + 2,
      "d": i + 3,
      "e": i + 4
    };

    let checksum = checksum + h["a"] + h["c"] + h["e"];
    let i = i + 1;
  };

  let round = round + 1;
};

puts("profile =", "hash-heavy");
puts("work =", WORK);
puts("rounds =", ROUNDS);
puts("checksum =", checksum);
