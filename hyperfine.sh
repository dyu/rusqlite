#!/bin/sh

#set -e

CURRENT_DIR=$PWD
# locate
if [ -z "$BASH_SOURCE" ]; then
    SCRIPT_DIR=`dirname "$(readlink -f $0)"`
elif [ -e '/bin/zsh' ]; then
    F=`/bin/zsh -c "print -lr -- $BASH_SOURCE(:A)"`
    SCRIPT_DIR=`dirname $F`
elif [ -e '/usr/bin/realpath' ]; then
    F=`/usr/bin/realpath $BASH_SOURCE`
    SCRIPT_DIR=`dirname $F`
else
    F=$BASH_SOURCE
    while [ -h "$F" ]; do F="$(readlink $F)"; done
    SCRIPT_DIR=`dirname $F`
fi
# change pwd
cd $SCRIPT_DIR

TARGETS='
bench_inserts
bench_inserts_exec
'

for F in $TARGETS; do
    if [ "$BUILD" = '1' ] || [ ! -e "target/release/$F" ]; then
        cargo build --release --package $F
    fi
done

BENCH=1 hyperfine \
--prepare 'rm target/db.sqlite*' \
--sort command -w 0 -r 1 \
'./target/release/bench_inserts 0' \
'./target/release/bench_inserts 1' \
'./target/release/bench_inserts_exec 0' \
'./target/release/bench_inserts_exec 1' && \
echo
