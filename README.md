 A simplified version of Bitcoin.
https://github.com/Jeiwan/blockchain_go

Part2:
At first, I use the digest(data) to get hash, but I get a really big number when I use the hash to be hash\_int. The number is always bigger than target.
`let hash_int = BigUint::from_bytes_be(&hash);`
After I switch to use hasher.update(data) to get hash, all work well.
I don't know what's wrong.

