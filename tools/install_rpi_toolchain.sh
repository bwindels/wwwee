mkdir -p cross_compilers/
cd cross_compilers/
wget https://github.com/raspberrypi/tools/archive/master.zip
unzip master.zip
rm master.zip
mv tools-master/ rpi/