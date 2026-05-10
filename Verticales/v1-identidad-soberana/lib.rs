#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    /// Configuración del módulo IDS.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        
        /// El nivel de encriptación exigido (ej. Post-Cuántica PQC).
        #[pallet::constant]
        type MinimumEncryptionLevel: Get<u8>;
    }

    /// Estructura de la Identidad Ciudadana
    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct SovereignIdentity<AccountId> {
        pub owner: AccountId,
        pub zkp_hash: [u8; 32], // Hash de la Prueba de Conocimiento Cero
        pub neuro_consent_active: bool, // Protección de Inviolabilidad Cognitiva
        pub fea_pub_key: [u8; 64], // Clave Pública de Firma Electrónica Avanzada
    }

    /// Registro principal: Mapea una cuenta a su Identidad Soberana.
    #[pallet::storage]
    #[pallet::getter(fn identity_registry)]
    pub type IdentityRegistry<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        SovereignIdentity<T::AccountId>,
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        IdentityRegistered(T::AccountId),
        NeuroConsentRevoked(T::AccountId),
        FeaKeyUpdated(T::AccountId),
    }

    #[pallet::error]
    pub enum Error<T> {
        IdentityAlreadyExists,
        IdentityDoesNotExist,
        InvalidZKP,
        UnauthorizedAccess,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Registra una nueva Identidad Soberana desde la App Móvil.
        #[pallet::call_index(0)]
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn register_identity(
            origin: OriginFor<T>,
            zkp_hash: [u8; 32],
            fea_pub_key: [u8; 64],
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(!IdentityRegistry::<T>::contains_key(&who), Error::<T>::IdentityAlreadyExists);

            let new_identity = SovereignIdentity {
                owner: who.clone(),
                zkp_hash,
                neuro_consent_active: true, // Consentimiento activo por defecto
                fea_pub_key,
            };

            IdentityRegistry::<T>::insert(&who, new_identity);
            Self::deposit_event(Event::IdentityRegistered(who));

            Ok(())
        }

        /// Revoca el acceso a atributos cognitivos y de datos (Filtro Soberano).
        #[pallet::call_index(1)]
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
        pub fn revoke_neuro_consent(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            IdentityRegistry::<T>::try_mutate(&who, |maybe_id| -> DispatchResult {
                let id = maybe_id.as_mut().ok_or(Error::<T>::IdentityDoesNotExist)?;
                id.neuro_consent_active = false;
                Ok(())
            })?;

            Self::deposit_event(Event::NeuroConsentRevoked(who));
            Ok(())
        }
    }
}