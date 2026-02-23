let isPrime = fn(n) {
  if (n <= 1) {
    false;
  } else {
    let d = 2;
    let composite = false;

    while (d * d <= n) {
      if ((n / d) * d == n) {
        let composite = true;
        let d = n;
      } else {
        let d = d + 1;
      }
    };

    if (composite) {
      false;
    } else {
      true;
    }
  }
};

let LIMIT = 1000;
let REPEATS = 2;

let round = 0;
let totalPrimeCount = 0;
let totalPrimeSum = 0;

let smallCount = 0;
let mediumCount = 0;
let largeCount = 0;

while (round < REPEATS) {
  let n = 1;
  while (n <= LIMIT) {
    if (isPrime(n)) {
      let totalPrimeCount = totalPrimeCount + 1;
      let totalPrimeSum = totalPrimeSum + n;

      if (n <= 100) {
        let smallCount = smallCount + 1;
      } else if (n <= 500) {
        let mediumCount = mediumCount + 1;
      } else if (n >= 501) {
        let largeCount = largeCount + 1;
      } else {
        let smallCount = smallCount;
      }
    } else {
      let totalPrimeCount = totalPrimeCount;
    }

    let n = n + 1;
  };

  let round = round + 1;
};

puts("limit =", LIMIT);
puts("repeats =", REPEATS);
puts("primeCount =", totalPrimeCount);
puts("primeSum =", totalPrimeSum);
puts("smallPrimes =", smallCount);
puts("mediumPrimes =", mediumCount);
puts("largePrimes =", largeCount);