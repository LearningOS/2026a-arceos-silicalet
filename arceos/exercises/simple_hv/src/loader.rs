use std::fs::File;
use std::io::{self, Read};

use axhal::paging::MappingFlags;
use axhal::mem::{PAGE_SIZE_4K, phys_to_virt};
use axmm::AddrSpace;
use crate::VM_ENTRY;

pub fn load_vm_image(fname: &str, uspace: &mut AddrSpace) -> io::Result<()> {
    let mut file = File::open(fname)?;
    uspace.map_alloc(
        VM_ENTRY.into(),
        PAGE_SIZE_4K,
        MappingFlags::READ | MappingFlags::WRITE | MappingFlags::EXECUTE | MappingFlags::USER,
        true,
    ).unwrap();

    let (paddr, _, _) = uspace
        .page_table()
        .query(VM_ENTRY.into())
        .unwrap_or_else(|_| panic!("Mapping failed for segment: {:#x}", VM_ENTRY));

    // The mapping above allocated and zeroed this page for the guest image.
    let page = unsafe {
        core::slice::from_raw_parts_mut(phys_to_virt(paddr).as_mut_ptr(), PAGE_SIZE_4K)
    };
    let mut loaded = 0;
    while loaded < page.len() {
        let n = file.read(&mut page[loaded..])?;
        if n == 0 {
            return if loaded == 0 { Err(io::Error::InvalidData) } else { Ok(()) };
        }
        loaded += n;
    }
    if file.read(&mut [0])? != 0 {
        return Err(io::Error::InvalidData);
    }
    Ok(())
}
