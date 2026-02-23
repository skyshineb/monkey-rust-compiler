let fib = fn(n) {
  if (n == 0) {
    0;
  } else {
    if (n == 1) {
      1;
    } else {
      fib(n - 1) + fib(n - 2);
    }
  }
};

let N = 28;
let ROUNDS = 5;

let i = 0;
let checksum = 0;

while (i < ROUNDS) {
  let checksum = checksum + fib(N);
  let i = i + 1;
};

puts("fib N =", N);
puts("rounds =", ROUNDS);
puts("checksum =", checksum);