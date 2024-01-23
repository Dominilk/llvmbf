set -e
apt update
apt install -y lsb-release wget software-properties-common gnupg

apt install libzstd-dev


wget -O /tmp/llvm-inst.sh https://apt.llvm.org/llvm.sh
chmod +x /tmp/llvm-inst.sh
sudo /tmp/llvm-inst.sh 17