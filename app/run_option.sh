if [ "$1" = "run" ]; then
    cargo run --release scences/test.json
elif [ "$1" = "check" ]; then
    echo "lll"
    cargo check scences/test.json
fi