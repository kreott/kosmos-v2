# Testing

You will have to create the .img file yourself

run ``dd if=/dev/zero of=qemu-fat32.img bs=1M count=256`` to create the image,
then ``mkfs.fat -F 32 qemu-fat32.img`` to format it. You can change the filename if you'd like, however you will have to edit the launch commands in the root Cargo.toml to load the image into qemu.