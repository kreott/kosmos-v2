# Kosmos
Tiny passion project that started as just following along Philipp Opperman's [Writing an OS in Rust](https://os.phil-opp.com/) blog!
If you are interested in OS development I highly suggest you check it out, it's amazing.

Currently only supports legacy BIOS and not UEFI, but runnable in SeaBIOS on UEFI hardware possibly idk i havent tried that yet

# Testing

You will have to create the .img files yourself

- run ``dd if=/dev/zero of=qemu-ata-fat32.img bs=1M count=64 && dd if=/dev/zero of=qemu-ahci-fat32.img bs=1M count=64`` to create the images,
- then ``mkfs.fat -F 32 -s 1 -S 512 tests/qemu-ata-fat32.img && mkfs.fat -F 32 -s 1 -S 512 tests/qemu-ahci-fat32.img`` to format them.

You can change the filenames if you'd like, however you will have to edit the run-args and test-args fields in the root Cargo.toml to load them into qemu properly.
