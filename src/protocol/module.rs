#[derive(Debug, Clone, Copy, PartialEq, defmt::Format)]
pub enum FirmwareType {
    LD, // ?
    AP2, // Primary Firmware
    AP1 // Backup Firmware
}

#[derive(Debug, Clone, Copy, PartialEq, defmt::Format)]
pub enum ModuleSetCommand {
    ShowLogo,                      
    UpdateBootLogo { slice: u8 },  
    SetBrightness { value: u8 },   
    SetIdleTime { seconds: i32 },  
}

#[derive(Debug, Clone, Copy, PartialEq, defmt::Format)]
pub enum ModuleGetCommand {
    GetFirmwareVersion(FirmwareType),
    GetUnitSerialNumber,             
    GetIdleTime,                     
}