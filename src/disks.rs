use anyhow::{anyhow, Result};

use disk_types::FileSystem;
use fstab_generate::BlockInfo;
use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

const EFI_DETECT_PATH: &str = "/sys/firmware/efi";
pub(crate) const ALLOWED_FS_TYPE: &[&str] = &["ext4", "xfs", "btrfs", "f2fs"];
const DEFAULT_FS_TYPE: &str = "ext4";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Partition {
    pub path: Option<PathBuf>,
    pub parent_path: Option<PathBuf>,
    pub fs_type: Option<String>,
    pub size: u64,
}

#[inline]
pub fn is_efi_booted() -> bool {
    Path::new(EFI_DETECT_PATH).is_dir()
}

pub fn get_recommended_fs_type(type_: &str) -> &str {
    for i in ALLOWED_FS_TYPE {
        if *i == type_ {
            return i;
        }
    }

    DEFAULT_FS_TYPE
}

pub fn format_partition(partition: &Partition) -> Result<()> {
    let default_fs = DEFAULT_FS_TYPE.to_owned();
    let fs_type = partition.fs_type.as_ref().unwrap_or(&default_fs);
    let mut command = Command::new(format!("mkfs.{}", fs_type));
    let cmd;
    let output;
    if fs_type == "ext4" {
        cmd = command.arg("-Fq");
    } else if fs_type == "fat32" {
        cmd = command.arg("-F32");
    } else {
        cmd = command.arg("-f");
    }
    output = cmd
        .arg(
            partition
                .path
                .as_ref()
                .ok_or_else(|| anyhow!("Path not found"))?,
        )
        .output()?;
    if !output.status.success() {
        return Err(anyhow!(
            "Failed to create filesystem: \n{}\n{}",
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        ));
    }

    Ok(())
}

pub fn fill_fs_type(part: &Partition, use_ext4: bool) -> Partition {
    let mut new_part = part.clone();
    let new_fs_type: String;
    if let Some(fs_type) = new_part.fs_type.clone() {
        if !use_ext4 {
            new_fs_type = get_recommended_fs_type(&fs_type).to_string();
        } else {
            new_fs_type = DEFAULT_FS_TYPE.to_string();
        }
    } else {
        new_fs_type = DEFAULT_FS_TYPE.to_string();
    }
    new_part.fs_type = Some(new_fs_type);

    new_part
}

pub fn find_esp_partition(device_path: &Path) -> Result<Partition> {
    let mut device = libparted::Device::get(device_path)?;
    if let Ok(disk) = libparted::Disk::new(&mut device) {
        for mut part in disk.parts() {
            if part.num() < 0 {
                continue;
            }
            if part.get_flag(libparted::PartitionFlag::PED_PARTITION_ESP) {
                let fs_type;
                if let Ok(type_) = part.get_geom().probe_fs() {
                    fs_type = Some(type_.name().to_owned());
                } else {
                    fs_type = None;
                }
                let path = part
                    .get_path()
                    .ok_or_else(|| anyhow!("Unable to get the device file for ESP partition"))?;
                return Ok(Partition {
                    path: Some(path.to_owned()),
                    parent_path: None,
                    size: 0,
                    fs_type,
                });
            }
        }
    }

    Err(anyhow!("ESP partition not found."))
}

pub fn list_partitions() -> Vec<Partition> {
    let mut partitions: Vec<Partition> = Vec::new();
    for mut device in libparted::Device::devices(true) {
        let device_path = device.path().to_owned();
        let sector_size: u64 = device.sector_size();
        if let Ok(disk) = libparted::Disk::new(&mut device) {
            for mut part in disk.parts() {
                if part.num() < 0 {
                    continue;
                }
                let fs_type;
                let geom_length: i64 = part.geom_length();
                let part_length = if geom_length < 0 {
                    0
                } else {
                    geom_length as u64
                };
                if let Ok(type_) = part.get_geom().probe_fs() {
                    fs_type = Some(type_.name().to_owned());
                } else {
                    fs_type = None;
                }
                partitions.push(Partition {
                    path: part.get_path().map(|path| path.to_owned()),
                    parent_path: Some(device_path.clone()),
                    size: sector_size * part_length,
                    fs_type,
                });
            }
        }
    }

    partitions
}

pub fn fstab_entries(partition: &Partition, mount_path: &Path) -> Result<OsString> {
    let target = partition.path.as_ref().unwrap();
    let fs_type = partition
        .fs_type
        .as_ref()
        .ok_or_else(|| anyhow!("Could get partition Object!"))?;
    let (fs_type, option) = match fs_type.as_str() {
        "fat32" => (FileSystem::Fat32, "defaults"),
        "ext4" => (FileSystem::Ext4, "defaults"),
        "btrfs" => (FileSystem::Btrfs, "defaults"),
        "xfs" => (FileSystem::Xfs, "defaults"),
        "f2fs" => (FileSystem::F2fs, "defaults"),
        "swap" => (FileSystem::Swap, "sw"),
        _ => return Err(anyhow!("Unsupport fs type!")),
    };
    let root_id = BlockInfo::get_partition_id(target, fs_type)
        .ok_or_else(|| anyhow!("Could not get partition uuid!"))?;
    let root = BlockInfo::new(root_id, fs_type, Some(mount_path), option);
    let fstab = &mut OsString::new();
    root.write_entry(fstab);

    Ok(fstab.to_owned())
}

#[test]
fn test_fs_recommendation() {
    assert_eq!(get_recommended_fs_type("btrfs"), "btrfs");
    assert_eq!(get_recommended_fs_type("ext2"), "ext4");
}
