dd if=/dev/zero of=fat_1.img count=1024
dd if=/dev/zero of=fat_2.img count=1024
mkfs.vfat fat_1.img
mkfs.vfat fat_2.img
