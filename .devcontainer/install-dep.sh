set -e
apt update
apt install -y lsb-release wget software-properties-common gnupg

apt install libzstd-dev


wget -O /tmp/llvm-inst.sh https://apt.llvm.org/llvm.sh
chmod +x /tmp/llvm-inst.sh
/tmp/llvm-inst.sh 17

# install libpolly after llvm installation so that apt sources have llvm repo.
apt install libpolly-17-dev