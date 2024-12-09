use std::{
    collections::{VecDeque}, path::{Path, PathBuf}
};

#[derive(Debug, Clone)]
struct AllocatedBlocks {
    disk_offset: usize,
    id: usize,
    length: usize,
}

#[derive(Debug, Clone)]
struct FreeBlocks {
    _disk_offset: usize,
    length: usize
}

#[derive(Debug, Clone)]
struct DiskMap {
    // the raw alternating fs blocks / free
    allocs: Vec<AllocatedBlocks>,
    free_list: Vec<FreeBlocks>,
}

impl DiskMap {
    fn from_raw(raw: Vec<u8>) -> Self {
        let mut free_list: Vec<FreeBlocks> = Vec::new();
        let mut allocs: Vec<AllocatedBlocks> = Vec::new();
        let mut block_offset = 0;
        let mut block_id = 0;
        for (raw_idx, len) in raw.iter().enumerate() {
            if raw_idx % 2 == 0 {
                allocs.push(AllocatedBlocks {
                    disk_offset: block_offset,
                    id: block_id,
                    length: *len as usize,
                });
                block_id += 1;
            } else {
                free_list.push(FreeBlocks {
                    _disk_offset: block_offset,
                    length: *len as usize,
                });
            }
            block_offset += *len as usize;
        }
        DiskMap {
            allocs,
            free_list,
        }
    }
}

fn parse_diskmap<P: AsRef<Path>>(path: P) -> anyhow::Result<DiskMap> {
    let full_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("inputs")
        .join(path);
    let contents = std::fs::read_to_string(full_path)?;
    let trimmed = contents.trim();
    let diskmap_raw = trimmed
        .bytes()
        .map(|c| {
            if c < b'0' || c > b'9' {
                panic!("Unexpected char in diskmap: '{c:?}'");
            }
            c - b'0'
        })
        .collect();
    Ok(DiskMap::from_raw(diskmap_raw))
}

fn compact_disk(diskmap: &DiskMap) -> Vec<usize> {
    let mut allocs = VecDeque::from_iter(diskmap.allocs.iter().cloned());
    let mut frees = VecDeque::from_iter(diskmap.free_list.iter().cloned());

    let mut compacted = Vec::new();
    while !frees.is_empty() && !allocs.is_empty() {
        let idx = compacted.len();
        if allocs[0].disk_offset == idx {
            let alloc = allocs.pop_front().unwrap();
            compacted.extend(itertools::repeat_n(alloc.id, alloc.length));
            continue;
        }

        // fill free space
        let alloc = allocs.back_mut().unwrap();
        let free = frees.front_mut().unwrap();
        while alloc.length > 0 && free.length > 0 {
            compacted.push(alloc.id);
            alloc.length -= 1;
            free.length -= 1;
        }
        if alloc.length == 0 {
            allocs.pop_back();
        }
        if free.length == 0 {
            frees.pop_front();
        }
    }

    compacted
}

fn checksum(disk: &[usize]) -> usize {
    disk.iter().enumerate().map(|(i, id)| i * *id).sum()
}

fn main() -> anyhow::Result<()> {
    let diskmap = parse_diskmap("d9-p1.txt")?;
    // println!("diskmap: {:?}", diskmap);
    let compacted = compact_disk(&diskmap);
    // println!("Compacted: {compacted:?}");
    println!("Checksum: {}", checksum(&compacted));
    Ok(())
}
