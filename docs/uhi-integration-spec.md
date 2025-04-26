# UHI Integration Technical Specification

This document outlines the integration of the Unified Health Interface (UHI) protocol into the Chiti Identity System, focusing on appointment booking as the primary implementation flow.

## 1. UHI Protocol Overview

The UHI Protocol is an adaptation of the Beckn protocol (version 0.9.3) for healthcare services. It follows the same request-callback pattern and core message structure, with healthcare-specific schemas and flows.

### 1.1 Key Characteristics

- **Protocol Version**: UHI Protocol specification 0.0.1
- **Domain**: `nic2004:85110` (Healthcare domain identifier)
- **Transaction Flow**: Search → Select → Init → Confirm → Status (with corresponding callbacks)
- **Architecture**: UHI Network comprises End User Applications (EUAs) and Health Service Provider (HSP) applications

### 1.2 Core Components

- **UHI Gateway**: Facilitates message routing between participants
- **UHI Registry**: Maintains registry of healthcare providers and services
- **EUA**: End user-facing applications for patients
- **HSP**: Healthcare service provider applications

## 2. System Architecture for UHI Integration

### 2.1 Healthcare Service Handler

We will implement a `HealthcareServiceHandler` that implements the `ServiceHandler` trait, specifically tailored for UHI protocol operations:

```rust
pub struct HealthcareServiceHandler {
    /// Configuration for healthcare services
    config: HealthcareConfig,
    /// Validator for UHI-specific schemas
    validator: UHIValidator,
    /// Connector for external EHR systems
    ehr_connector: EHRConnector,
    /// Registry client for UHI Network Registry
    uhi_registry: UHIRegistryClient,
}

impl ServiceHandler for HealthcareServiceHandler {
    type State = AppointmentState;
    
    fn domain(&self) -> &str {
        "nic2004:85110"
    }
    
    async fn handle_action(
        &self, 
        action: BecknAction, 
        context: ActionContext
    ) -> Result<StateChange, ServiceError>;
    
    fn validate_payload(
        &self, 
        action: BecknAction, 
        payload: &serde_json::Value
    ) -> Result<(), ValidationError>;
}
```

### 2.2 Healthcare Namespace and Data Model

We will define a healthcare-specific namespace and path structure within the Willow data model:

```rust
/// Healthcare service namespace configuration
pub const HEALTHCARE_NAMESPACE_CONFIG: NamespaceConfig = NamespaceConfig {
    shortname: "healthcare",
    namespace_type: NamespaceType::Owned,
    service_type: ServiceType::Healthcare,
    description: Some("UHI Protocol healthcare service namespace"),
};

/// Healthcare subspace path structure
pub mod path {
    pub const PROVIDERS: &str = "/providers";
    pub const PRACTITIONERS: &str = "/practitioners";
    pub const APPOINTMENTS: &str = "/appointments";
    pub const CONSULTATIONS: &str = "/consultations";
    pub const MEDICAL_RECORDS: &str = "/medical_records";
    
    pub mod appointment {
        pub const STATE: &str = "/state";
        pub const CATALOG: &str = "/catalog";
        pub const BOOKING: &str = "/booking";
        pub const PAYMENT: &str = "/payment";
    }
}
```

### 2.3 Healthcare Integration Components

Three new components will be added to facilitate healthcare-specific functionality:

1. **EHR Connector**: Integrates with external Electronic Health Record systems
   ```rust
   pub struct EHRConnector {
       connectors: HashMap<String, Box<dyn EHRSystemConnector>>,
       config: EHRConnectorConfig,
   }
   
   pub trait EHRSystemConnector: Send + Sync {
       async fn fetch_patient_records(&self, patient_id: &str) -> Result<PatientRecords, EHRError>;
       async fn sync_appointment(&self, appointment: &Appointment) -> Result<(), EHRError>;
       async fn verify_provider(&self, provider_id: &str) -> Result<ProviderDetails, EHRError>;
   }
   ```

2. **UHI Schema Validator**: Validates UHI-specific message formats
   ```rust
   pub struct UHIValidator {
       schemas: HashMap<BecknAction, JsonSchema>,
   }
   
   impl UHIValidator {
       pub fn validate_healthcare_specific(&self, action: BecknAction, payload: &serde_json::Value) -> Result<(), ValidationError>;
       pub fn validate_appointment(&self, appointment: &Appointment) -> Result<(), ValidationError>;
       pub fn validate_practitioner(&self, practitioner: &Practitioner) -> Result<(), ValidationError>;
   }
   ```

3. **Healthcare Entities**: Models for healthcare domain
   ```rust
   pub struct HealthcareEntities {
       provider_store: Arc<dyn ProviderStore>,
       practitioner_store: Arc<dyn PractitionerStore>,
       appointment_store: Arc<dyn AppointmentStore>,
   }
   ```

## 3. Healthcare Data Models

### 3.1 Appointment Transaction State

The `AppointmentState` type will implement the `BecknTransactionState` trait to track appointment booking flow:

```rust
pub struct AppointmentState {
    transaction_id: String,
    status: TransactionStatus,
    domain: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    
    // Healthcare-specific fields
    appointment_id: Option<String>,
    provider: Option<Provider>,
    practitioner: Option<Practitioner>,
    patient: Option<Patient>,
    appointment_time: Option<AppointmentTime>,
    service_type: Option<String>, // e.g., "DIGITAL-OPD"
    payment_details: Option<PaymentDetails>,
    ehr_sync_status: Option<EHRSyncStatus>,
}

impl BecknTransactionState for AppointmentState {
    fn transaction_id(&self) -> &str {
        &self.transaction_id
    }
    
    fn status(&self) -> &TransactionStatus {
        &self.status
    }
    
    fn update_status(&mut self, status: TransactionStatus) {
        self.status = status;
        self.touch();
    }
    
    fn domain(&self) -> &str {
        &self.domain
    }
    
    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    
    fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
    
    fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}
```

### 3.2 Healthcare Domain Entities

```rust
pub struct Provider {
    id: String,
    name: String,
    description: Option<String>,
    contact: Contact,
    address: Address,
    categories: Vec<Category>,
    services: Vec<Service>,
    practitioners: Vec<String>, // Practitioner IDs
    uhi_id: Option<String>,     // UHI-specific identifier
}

pub struct Practitioner {
    id: String,
    name: String,
    gender: Option<String>,
    specialization: Vec<String>,
    qualifications: Vec<String>,
    experience_years: Option<u32>,
    languages: Vec<String>,
    available_slots: Vec<TimeSlot>,
    uhi_id: Option<String>,     // UHI-specific identifier
}

pub struct Patient {
    id: String,
    health_id: Option<String>,  // ABHA/Health ID
    phr_address: Option<String>, // Personal Health Record address
    name: String,
    gender: Option<String>,
    dob: Option<NaiveDate>,
    contact: Contact,
}

pub struct AppointmentTime {
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    time_zone: String,
}

pub struct Service {
    id: String,
    name: String,
    category_id: String,
    description: Option<String>,
    price: Price,
    duration_minutes: u32,
}

pub enum EHRSyncStatus {
    NotSynced,
    SyncInProgress,
    SyncComplete,
    SyncFailed,
}
```

## 4. UHI API Implementation

The UHI API endpoints will be implemented in accordance with the UHI Protocol specification, focusing on the appointment booking flow:

### 4.1 API Endpoints

1. **Search** (`/search`): Find healthcare providers, practitioners, or services
2. **Select** (`/select`): Choose a specific appointment slot/practitioner
3. **Init** (`/init`): Provide patient details and initiate booking
4. **Confirm** (`/confirm`): Finalize the appointment booking
5. **Status** (`/status`): Check appointment status

### 4.2 Callback Endpoints

1. **On Search** (`/on_search`): Return catalog of available slots/practitioners
2. **On Select** (`/on_select`): Respond with quote for selected slot
3. **On Init** (`/on_init`): Respond with draft booking details
4. **On Confirm** (`/on_confirm`): Respond with confirmed booking details
5. **On Status** (`/on_status`): Respond with current appointment status

### 4.3 UHI Appointment Booking Flow

```
┌─────────┐                                     ┌─────────┐
│ Patient │                                     │Provider │
│   EUA   │                                     │   HSP   │
└────┬────┘                                     └────┬────┘
     │                                               │
     │ search (find doctors/slots)                   │
     ├───────────────────────────────────────────────► 
     │                                               │
     │                 on_search (catalog)           │
     ◄───────────────────────────────────────────────┤
     │                                               │
     │ select (choose slot)                          │
     ├───────────────────────────────────────────────►
     │                                               │
     │                on_select (quote)              │
     ◄───────────────────────────────────────────────┤
     │                                               │
     │ init (provide patient details)                │
     ├───────────────────────────────────────────────►
     │                                               │
     │              on_init (draft booking)          │
     ◄───────────────────────────────────────────────┤
     │                                               │
     │ confirm (accept booking)                      │
     ├───────────────────────────────────────────────►
     │                                               │
     │           on_confirm (confirmation)           │
     ◄───────────────────────────────────────────────┤
     │                                               │
     │ status (check appointment)                    │
     ├───────────────────────────────────────────────►
     │                                               │
     │         on_status (appointment details)       │
     ◄───────────────────────────────────────────────┤
     │                                               │
```

## 5. Integration with External Healthcare Systems

### 5.1 EHR Integration

```rust
pub struct EHRConnectorConfig {
    systems: Vec<EHRSystemConfig>,
    default_timeout_seconds: u64,
    retry_attempts: u32,
}

pub struct EHRSystemConfig {
    id: String,
    name: String,
    base_url: String,
    auth_type: EHRAuthType,
    credentials: EHRCredentials,
    connection_timeout_seconds: u64,
}

pub enum EHRAuthType {
    OAuth2,
    APIKey,
    BasicAuth,
    Custom(String),
}

impl EHRConnector {
    pub async fn new(config: EHRConnectorConfig) -> Result<Self, EHRError> {
        // Initialize connector with supported EHR systems
    }
    
    pub async fn get_patient_by_health_id(&self, health_id: &str) -> Result<Patient, EHRError> {
        // Fetch patient details from appropriate EHR system
    }
    
    pub async fn sync_appointment_to_ehr(
        &self, 
        appointment: &AppointmentState,
        ehr_system_id: &str
    ) -> Result<EHRSyncStatus, EHRError> {
        // Sync appointment details to specified EHR system
    }
}
```

### 5.2 UHI Registry Integration

```rust
pub struct UHIRegistryClient {
    registry_url: String,
    client: reqwest::Client,
    identity: Arc<Identity>,
}

impl UHIRegistryClient {
    pub async fn new(
        registry_url: String, 
        identity: Arc<Identity>
    ) -> Result<Self, RegistryError> {
        // Initialize UHI registry client
    }
    
    pub async fn lookup_provider(
        &self,
        provider_id: &str
    ) -> Result<ProviderDetails, RegistryError> {
        // Look up provider details in UHI registry
    }
    
    pub async fn lookup_practitioner(
        &self,
        practitioner_id: &str
    ) -> Result<PractitionerDetails, RegistryError> {
        // Look up practitioner details in UHI registry
    }
    
    pub async fn register_as_participant(
        &self,
        role: UHIRole,
        domains: Vec<String>,
    ) -> Result<RegistrationResponse, RegistryError> {
        // Register this node as a UHI network participant
    }
}

pub enum UHIRole {
    EUA,  // End User Application
    HSP,  // Health Service Provider
    Gateway,
}
```

## 6. Security and Compliance

### 6.1 Healthcare Data Security

All healthcare data will be encrypted at rest and in transit. The `access` module will be extended to implement healthcare-specific access controls:

```rust
pub struct HealthcareCapabilityPolicy {
    role_based_rules: HashMap<String, Vec<HealthcareCapability>>,
    operation_policies: Vec<HealthcareOperationPolicy>,
}

pub enum HealthcareCapability {
    ViewProviders,
    ViewPractitioners,
    BookAppointment,
    ViewAppointment,
    CancelAppointment,
    AccessMedicalRecords,
    UpdateMedicalRecords,
    // Other healthcare-specific capabilities
}

impl AccessManager {
    pub fn create_healthcare_capability(
        &self,
        patient_id: &str,
        provider_id: &str,
        capabilities: Vec<HealthcareCapability>,
        expires_at: Option<DateTime<Utc>>
    ) -> Result<StoredCapability, AccessError> {
        // Create healthcare-specific capability
    }
}
```

### 6.2 Compliance with Healthcare Standards

The implementation will adhere to healthcare data standards and regulations:

- Patient consent management for data sharing
- Audit logging of all healthcare data access
- Data minimization principles
- Secure transmission of Protected Health Information (PHI)

## 7. Testing Strategy

### 7.1 UHI Protocol Conformance Testing

```rust
#[cfg(test)]
mod uhi_tests {
    #[tokio::test]
    async fn test_appointment_booking_flow() {
        // Test complete appointment booking flow
    }
    
    #[tokio::test]
    async fn test_practitioner_discovery() {
        // Test search and discovery of healthcare practitioners
    }
    
    #[tokio::test]
    async fn test_ehr_integration() {
        // Test integration with mock EHR system
    }
}
```

### 7.2 Mock Healthcare Services

For testing, we'll implement mock healthcare services:

```rust
pub struct MockHealthcareProvider {
    id: String,
    practitioners: Vec<MockPractitioner>,
    services: Vec<MockService>,
}

pub struct MockPractitioner {
    id: String,
    name: String,
    specialization: String,
    available_slots: Vec<TimeSlot>,
}

impl MockHealthcareProvider {
    pub fn handle_search(&self, intent: &Intent) -> Vec<Catalog> {
        // Mock search implementation
    }
    
    pub fn handle_select(&self, order: &Order) -> Order {
        // Mock select implementation
    }
    
    // Other mock handlers
}
```

## 8. Deployment Considerations

### 8.1 Healthcare-Specific Configuration

```toml
[uhi]
domain = "nic2004:85110"
registry_url = "https://registry.uhi.example.org/v1"
api_port = 8080
callback_url = "https://example.com/uhi/callback"

[uhi.ehr]
enabled = true
default_timeout_seconds = 30
retry_attempts = 3

[[uhi.ehr.systems]]
id = "system1"
name = "Hospital EHR System"
base_url = "https://ehr.hospital.example.org/api"
auth_type = "OAuth2"
```

### 8.2 Monitoring and Observability

Healthcare-specific metrics and alerts will be added:

- Appointment booking success rate
- EHR integration status
- Healthcare data access patterns
- UHI protocol compliance metrics

## 9. Future Extensions

- **Teleconsultation**: Add video consultation capabilities
- **Lab Results**: Integration with diagnostic services
- **Prescription Management**: Digital prescription flow
- **Insurance Integration**: Claims processing
- **Emergency Services**: Urgent care booking

---

This technical specification outlines the core components and implementation strategy for integrating the UHI protocol into the Chiti Identity System, with a focus on appointment booking. The implementation leverages the existing Beckn-compatible service adaptation layer while adding healthcare-specific components and data models.