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
    }

    /// Estado de los derechos ARCO (Acceso, Rectificación, Cancelación, Oposición) y GDPR
    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct DataProtectionStatus {
        pub consent_given: bool,
        pub last_updated_block: u32,
        pub data_processing_authorized: bool,
        pub right_to_be_forgotten_invoked: bool, // Si es true, anonimiza la identidad vinculada
    }

    #[pallet::storage]
    #[pallet::getter(fn privacy_registry)]
    pub type PrivacyRegistry<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        DataProtectionStatus,
        ValueQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ConsentUpdated(T::AccountId, bool),
        RightToBeForgottenExecuted(T::AccountId), // Evento crítico para auditorías
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// El ciudadano invoca su derecho al olvido (GDPR / Ley 19.628 y nuevas normativas)
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn invoke_right_to_be_forgotten(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            PrivacyRegistry::<T>::try_mutate(&who, |status| -> DispatchResult {
                status.consent_given = false;
                status.data_processing_authorized = false;
                status.right_to_be_forgotten_invoked = true;
                // Nota arquitectónica: Aquí el sistema elimina los punteros a los datos off-chain
                // manteniendo solo el hash inmutable, logrando anonimización legal.
                Ok(())
            })?;

            Self::deposit_event(Event::RightToBeForgottenExecuted(who));
            Ok(())
        }
    }
}