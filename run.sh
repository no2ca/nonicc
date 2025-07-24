cargo run "$1" > tmp.s
gcc -o tmp tmp.s
./tmp
echo "OUTPUT: $?"
rm -f tmp tmp.s
exit 0