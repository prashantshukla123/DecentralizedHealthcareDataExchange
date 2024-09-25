#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Symbol, String, symbol_short};

// This structure will keep track of access status (permissions) and ownership of healthcare data.
#[contracttype]
#[derive(Clone)]
pub struct DataAccessStatus {
    pub granted: u64,    // Count of permissions granted to healthcare providers.
    pub pending: u64,    // Count of pending requests for data access.
    pub revoked: u64,    // Count of permissions revoked by patients.
    pub total: u64       // Total number of healthcare data records on the platform.
}

// Symbol to reference all healthcare data in the system.
const ALL_DATA: Symbol = symbol_short!("ALL_DATA");

// For mapping healthcare record IDs to their respective AdminControl (permission) struct.
#[contracttype]
pub enum Adminbook {
    Admincontrol(u64)
}

// Structure to control access to healthcare data, storing admin's control status and request ID.
#[contracttype]
#[derive(Clone)]
pub struct Admincontrol {
    pub record_id: u64,    // Unique record ID for each healthcare data.
    pub access_granted: bool,  // Whether the healthcare provider has access.
}

// For mapping healthcare data to its unique record ID.
#[contracttype]
pub enum HealthData {
    Data(u64)
}

// Constant for keeping track of the total count of healthcare data entries.
const COUNT_DATA: Symbol = symbol_short!("C_DATA");

// This structure stores healthcare data and details about the data owner (patient).
#[contracttype]
#[derive(Clone)]
pub struct HealthRecord {
    pub record_id: u64,
    pub patient_id: String,   // Patient ID or identification
    pub data_hash: String,    // Hash of the healthcare data for secure reference
    pub timestamp: u64,       // Time when the data was uploaded
    pub is_revoked: bool,     // Whether access has been revoked by the patient
}

#[contract]
pub struct HealthcareDataContract;

#[contractimpl]
impl HealthcareDataContract {
    // Function to create a new healthcare data record and return its unique ID
    pub fn create_data(env: Env, patient_id: String, data_hash: String) -> u64 {
        let mut count_data: u64 = env.storage().instance().get(&COUNT_DATA).unwrap_or(0);
        count_data += 1;

        // Check if this is a new data entry.
        let mut record = Self::view_data(env.clone(), count_data.clone());

        if record.is_revoked == false {
            let time = env.ledger().timestamp();

            // Initialize the health record
            record.patient_id = patient_id;
            record.data_hash = data_hash;
            record.timestamp = time;
            record.is_revoked = false;

            // Increment the count of total records
            let mut all_data = Self::view_all_data_status(env.clone());
            all_data.pending += 1;
            all_data.total = all_data.total + 1;

            record.record_id = all_data.total;

            // Store the new data record
            env.storage().instance().set(&HealthData::Data(record.record_id.clone()), &record);
            env.storage().instance().set(&ALL_DATA, &all_data);
            env.storage().instance().set(&COUNT_DATA, &count_data);

            log!(&env, "Healthcare data created with Record ID: {}", record.record_id);

            return record.record_id.clone();
        } else {
            panic!("Cannot create data, record is already revoked or pending.");
        }
    }

    // Function for a patient to revoke access to their healthcare data.
    pub fn revoke_access(env: Env, record_id: u64) {
        let mut record = Self::view_data(env.clone(), record_id.clone());

        if record.is_revoked == false {
            record.is_revoked = true;

            let mut all_data = Self::view_all_data_status(env.clone());
            all_data.revoked += 1;

            env.storage().instance().set(&HealthData::Data(record.record_id.clone()), &record);
            env.storage().instance().set(&ALL_DATA, &all_data);

            log!(&env, "Access to healthcare data Record ID: {} has been revoked.", record.record_id);
        } else {
            panic!("This data is already revoked or does not exist.");
        }
    }

    // Function for a healthcare provider to request access to a patient’s healthcare data.
    pub fn request_access(env: Env, record_id: u64) {
        let mut admin_control = Self::view_admin_control(env.clone(), record_id.clone());

        if admin_control.access_granted == false {
            let time = env.ledger().timestamp();

            admin_control.record_id = record_id;
            admin_control.access_granted = true;

            let mut all_data = Self::view_all_data_status(env.clone());
            all_data.granted += 1;
            all_data.pending -= 1;

            env.storage().instance().set(&Adminbook::Admincontrol(record_id.clone()), &admin_control);
            env.storage().instance().set(&ALL_DATA, &all_data);

            log!(&env, "Access granted to healthcare data Record ID: {}", record_id);
        } else {
            panic!("Access already granted or not allowed.");
        }
    }

    // View the current access status of all healthcare data records.
    pub fn view_all_data_status(env: Env) -> DataAccessStatus {
        env.storage().instance().get(&ALL_DATA).unwrap_or(DataAccessStatus {
            granted: 0,
            pending: 0,
            revoked: 0,
            total: 0,
        })
    }

    // View specific healthcare data using the record ID.
    pub fn view_data(env: Env, record_id: u64) -> HealthRecord {
        let key = HealthData::Data(record_id.clone());
        env.storage().instance().get(&key).unwrap_or(HealthRecord {
            record_id: 0,
            patient_id: String::from_str(&env, "Not Found"),
            data_hash: String::from_str(&env, "Not Found"),
            timestamp: 0,
            is_revoked: true,
        })
    }

    // View admin-controlled data, like access permissions.
    pub fn view_admin_control(env: Env, record_id: u64) -> Admincontrol {
        let ac_key = Adminbook::Admincontrol(record_id.clone());
        env.storage().instance().get(&ac_key).unwrap_or(Admincontrol {
            record_id: 0,
            access_granted: false,
        })
    }
}
