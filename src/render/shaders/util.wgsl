#define_import_path wormhole::util

fn extract_flag(data: u32, flag: u32) -> bool {
    return bool(data & flag);
}