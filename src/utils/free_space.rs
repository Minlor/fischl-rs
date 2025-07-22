use std::path::Path;
use sysinfo::Disks;

pub fn available(path: impl AsRef<Path>) -> Option<u64> {
    let mut disks = Disks::new_with_refreshed_list();

    disks.sort_by(|a, b| {
        let a = a.mount_point().as_os_str().len();
        let b = b.mount_point().as_os_str().len();
        a.cmp(&b).reverse()
    });

    let path = path.as_ref().canonicalize().unwrap_or_else(|_| path.as_ref().to_path_buf());

    for disk in disks.iter() {
        if path.starts_with(disk.mount_point()) {
            return Some(disk.available_space());
        }
    }
    None
}

pub fn is_same_disk(path1: impl AsRef<Path>, path2: impl AsRef<Path>) -> bool {
    let mut disks = Disks::new_with_refreshed_list();

    disks.sort_by(|a, b| {
        let a = a.mount_point().as_os_str().len();
        let b = b.mount_point().as_os_str().len();
        a.cmp(&b).reverse()
    });

    let path1 = path1.as_ref().canonicalize().unwrap_or_else(|_| path1.as_ref().to_path_buf());
    let path2 = path2.as_ref().canonicalize().unwrap_or_else(|_| path2.as_ref().to_path_buf());

    for disk in disks.iter() {
        let disk_path = disk.mount_point();
        if path1.starts_with(disk_path) && path2.starts_with(disk_path) {
            return true;
        }
    }
    false
}
