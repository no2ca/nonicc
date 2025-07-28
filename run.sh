cargo run "$1" > tmp.s
gcc -z noexecstack -o tmp tmp.s
cat tmp.s
./tmp
echo "OUTPUT: $?"
rm -f tmp tmp.s
exit 0