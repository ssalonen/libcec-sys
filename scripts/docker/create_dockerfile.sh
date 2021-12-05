cat << EOF > $1/Dockerfile
FROM rustembedded/cross:$1-$2

RUN apt-get remove --allow-remove-essential -y libudev1 udev libudev-dev || :
EOF
