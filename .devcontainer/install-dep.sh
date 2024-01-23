set -e
apt update
apt install -y lsb-release wget software-properties-common gnupg

apt install libpolly-17-dev libzstd-dev


wget -O /tmp/llvm-inst.sh https://apt.llvm.org/llvm.sh
chmod +x /tmp/llvm-inst.sh
/tmp/llvm-inst.sh 17