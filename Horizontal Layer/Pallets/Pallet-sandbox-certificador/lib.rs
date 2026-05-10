#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        
        /// Solo los auditores autorizados por la Cámara de Blockchain de Chile pueden certificar
        type ChamberAuditorOrigin: EnsureOrigin<Self::RuntimeOrigin>;
    }

    /// Estructura de Certificación de Sandbox
    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct CertificationRecord {
        pub target_hash: [u8; 32], // Hash del Smart Contract o RWA a certificar
        pub passed_stress_test: bool,
        pub passed_legal_compliance: bool, // ISO, GDPR, Ley Chilena
        pub auditor_signature: [u8; 64], // Firma PQC del auditor de la Cámara
    }

    #[pallet::storage]
    pub type CertifiedProducts<T: Config> = StorageMap<_, Blake2_128Concat, [u8; 32], CertificationRecord, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ProductStressTested([u8; 32], bool),
        OfficialCertificationGranted([u8; 32]), // Emitido por la Cámara
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Emite el certificado oficial tras pasar el Sandbox Regulatorio y Tecnológico
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn grant_chamber_certification(
            origin: OriginFor<T>,
            target_hash: [u8; 32],
            stress_test_passed: bool,
            legal_compliance_passed: bool,
            auditor_signature: [u8; 64],
        ) -> DispatchResult {
            // Requiere que quien ejecute esto sea un origin privilegiado (La Cámara)
            T::ChamberAuditorOrigin::ensure_origin(origin)?;

            let record = CertificationRecord {
                target_hash,
                passed_stress_test: stress_test_passed,
                passed_legal_compliance: legal_compliance_passed,
                auditor_signature,
            };

            CertifiedProducts::<T>::insert(&target_hash, record);
            
            if stress_test_passed && legal_compliance_passed {
                Self::deposit_event(Event::OfficialCertificationGranted(target_hash));
            } else {
                Self::deposit_event(Event::ProductStressTested(target_hash, false));
            }

            Ok(())
        }
    }
}