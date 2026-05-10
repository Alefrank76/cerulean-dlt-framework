#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    /// Registro de auditoría inmutable exigido por la norma ISO/IEC 27001:2022
    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct AuditLog {
        pub timestamp: u64,
        pub actor: [u8; 32], // Hash del actor/nodo
        pub action_type: u8, // 1: Acceso, 2: Modificación de clave, 3: Alerta de seguridad
        pub data_hash: [u8; 32],
    }

    #[pallet::storage]
    #[pallet::getter(fn security_logs)]
    pub type SecurityLogs<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64, // Log ID (secuencial)
        AuditLog,
        OptionQuery,
    >;

    #[pallet::storage]
    pub type LogCount<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        SecurityAuditLogged(u64, [u8; 32]), // Log ID, Hash del Actor
        CriticalSecurityAlert(T::AccountId, [u8; 32]),
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Registra un evento de seguridad de forma inmutable en la blockchain
        #[pallet::call_index(0)]
        #[pallet::weight(10_000 + T::DbWeight::get().writes(2))]
        pub fn register_audit_event(
            origin: OriginFor<T>,
            action_type: u8,
            data_hash: [u8; 32],
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            let log_id = LogCount::<T>::get().saturating_add(1);
            let timestamp = 0; // En producción se conecta al pallet de Timestamp de Substrate
            
            // Simulación de hash del actor
            let mut actor_hash = [0u8; 32];
            
            let log_entry = AuditLog {
                timestamp,
                actor: actor_hash,
                action_type,
                data_hash,
            };

            SecurityLogs::<T>::insert(log_id, log_entry);
            LogCount::<T>::put(log_id);

            Self::deposit_event(Event::SecurityAuditLogged(log_id, actor_hash));

            Ok(())
        }
    }
}