#!/bin/sh

make clean
cd enclave
make
cd ../
make


cd bin

rm -rf result.txt

for i in  4 8 16 32 64 128 256 512 1024 2048 # 4096
do
./app $i scan >> result.txt
done

echo "scan data:"
grep "dur=" result.txt | awk -F "=" '{print $2}'

cd -

