use super::Arch;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug)]
pub struct Image<'a> {
    pub arch: Arch,
    pub os: &'a str,
    pub prompt: &'a str,
    pub cdrom: &'a str,
    pub snapshot: &'a str,
    pub default_mem: &'a str,
    pub url: &'a str,
    pub extra_args: &'a [&'a str],
}

pub fn get_supported_image(name: &str) -> Image<'static> {
    match name {
        /*"i386_wheezy" => Image {
                arch: "i386",
                os:"linux-32-debian:3.2.0-4-686-pae",
                prompt:r#"root@debian-i386:.*# "#,
                qcow:"wheezy_panda2.qcow2", // Backwards compatability
                cdrom:"ide1-cd0",
                snapshot:"root",
                default_mem:"128M",
                url:"https://panda-re.mit.edu/qcows/linux/debian/7.3/x86/debian_7.3_x86.qcow",
                extra_args:"-display none"},

        "x86_64_wheezy" => Image {
                arch: "x86_64",
                os: "linux-64-debian:3.2.0-4-amd64",
                prompt: r#"root@debian-amd64:.*# "#,
                qcow="wheezy_x64.qcow2",// Backwards compatability 
                cdrom: "ide1-cd0",
                snapshot: "root",
                default_mem: "128M",
                url: "https://panda-re.mit.edu/qcows/linux/debian/7.3/x86_64/debian_7.3_x86_64.qcow",
                extra_args: "-display none"},

        "ppc_wheezy" => Image {
                arch: "ppc",
                os: "linux-64-debian:3.2.0-4-ppc-pae",
                prompt: r#"root@debian-powerpc:.*# "#,
                qcow="ppc_wheezy.qcow2",// Backwards compatability 
                cdrom: "ide1-cd0",
                default_mem: "128M",
                snapshot: "root",
                url: "https://panda-re.mit.edu/qcows/linux/debian/7.3/ppc/debian_7.3_ppc.qcow",
                extra_args: "-display none"},

        "arm_wheezy" => Image {
                arch: "arm",
                os: "linux-32-debian:3.2.0-4-versatile-arm",
                prompt: r#"root@debian-armel:.*# "#,
                qcow="arm_wheezy.qcow",// Backwards compatability 
                cdrom: "scsi0-cd2",
                default_mem: "128M",
                snapshot: "root",
                url: "https://panda-re.mit.edu/qcows/linux/debian/7.3/arm/debian_7.3_arm.qcow",
                extra_files=["vmlinuz-3.2.0-4-versatile', 'initrd.img-3.2.0-4-versatile"],
                extra_args: '-display none -M versatilepb -append "root=/dev/sda1" -kernel {DOT_DIR}/vmlinuz-3.2.0-4-versatile -initrd {DOT_DIR}/initrd.img-3.2.0-4-versatile'.format(DOT_DIR=VM_DIR)},

        "mips_wheezy" => Image {
                arch: "mips",
                os: "linux-64-debian:3.2.0-4-arm-pae", // XXX wrong
                prompt: r#"root@debian-mips:.*# "#,
                cdrom: "ide1-cd0",
                snapshot: "root",
                url: "https://panda-re.mit.edu/qcows/linux/debian/7.3/mips/debian_7.3_mips.qcow",
                default_mem: "1G",
                extra_files=['vmlinux-3.2.0-4-4kc-malta'],
                extra_args: '-M malta -kernel {DOT_DIR}/vmlinux-3.2.0-4-4kc-malta -append "root=/dev/sda1" -nographic'.format(DOT_DIR=VM_DIR)},

        "mipsel_wheezy":  Image {
                arch: "mipsel",
                os = "linux-32-debian:3.2.0-4-4kc-malta",
                prompt: r#"root@debian-mipsel:.*# "#,
                cdrom: "ide1-cd0",
                snapshot: "root",
                default_mem: "1G",
                url: "https://panda-re.mit.edu/qcows/linux/debian/7.3/mipsel/debian_7.3_mipsel.qcow",
                extra_files=["vmlinux-3.2.0-4-4kc-malta.mipsel",],
                extra_args: "-M malta -kernel {DOT_DIR}/vmlinux-3.2.0-4-4kc-malta.mipsel -append \"root=/dev/sda1\" -nographic"},

        // Ubuntu: x86/x86_64 support for 16.04, x86_64 support for 18.04
        "i386_ubuntu_1604" => Image {
                arch: "i386",
                os: "linux-32-ubuntu:4.4.200-170-generic", # Version.c is 200 but name is 4.4.0. Not sure why
                prompt: r#"root@instance-1:.*#"#,
                cdrom: "ide1-cd0",
                snapshot: "root",
                default_mem: "1024",
                url: "https://panda-re.mit.edu/qcows/linux/ubuntu/1604/x86/ubuntu_1604_x86.qcow",
                extra_args: "-display none"},

        // 'x86_64_ubuntu_1604' => Image { // XXX: This one is broken
        //         arch: "x86_64",
        //         os: "linux-64-ubuntu:4.4.0-180-pae",
        //         prompt: r#"root@instance-1:.*#"#,
        //         cdrom: "ide1-cd0",
        //         snapshot: "root",
        //         default_mem: "1024",
        //         url: "https://panda-re.mit.edu/qcows/linux/ubuntu/1604/x86_64/ubuntu_1604_x86_64.qcow",
        //         extra_files=['xenial-server-cloudimg-amd64-disk1.img',],
        //         extra_args: "-display none"},
*/
        "x86_64_ubuntu_1804" => Image {
                arch: Arch::x86_64,
                os: "linux-64-ubuntu:4.15.0-72-generic-noaslr-nokaslr",
                prompt: r#"root@ubuntu:.*#"#,
                cdrom: "ide1-cd0",
                snapshot: "root",
                default_mem: "1024",
                url: "https://panda-re.mit.edu/qcows/linux/ubuntu/1804/x86_64/bionic-server-cloudimg-amd64-noaslr-nokaslr.qcow2",
                extra_args: &["-display", "none"]},
        "x86_64" => get_supported_image("x86_64_ubuntu_1804"),
        _ => panic!("Unsupported image {}", name)
    }
}

fn panda_image_dir() -> PathBuf {
    let dir = dirs::home_dir().unwrap().join(".panda");

    if !dir.exists() {
        std::fs::create_dir(&dir).unwrap();
    }

    dir
}

// Given a generic name of a qcow or a path to a qcow, return the path. Downloads the qcow if it
// hasn't already been downloaded to ~/.panda/ yet.
pub fn get_generic_path(name: &str) -> PathBuf {
    let image = get_supported_image(name);
    let filename = image.url.split('/').last().unwrap();
    let path = panda_image_dir().join(filename);

    if !path.exists() {
        println!(
            "QCOW {} doesn't exist. Downloading from https://panda-re.mit.edu. Thanks MIT!",
            name
        );
        Command::new("wget")
            .args(&["--quiet", &image.url, "-O"])
            .arg(&path)
            .status()
            .unwrap();
    }

    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_install_qcow() {
        let x = get_generic_path("x86_64");

        assert!(x.exists());
    }
}
