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
        /// Vínculo con el módulo de Identidad (IDS) para validar la Firma Electrónica Avanzada (FEA).
        type IdentityValidator: crate::traits::SovereignIdentityVerifier<Self::AccountId>;
    }

    /// Estructura de un Activo del Mundo Real (RWA)
    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct RealWorldAsset {
        pub photo_hash: [u8; 32],        // Hash SHA-3 de la fotografía tomada en la App
        pub fea_signature: [u8; 64],     // Firma Electrónica Avanzada del ciudadano
        pub geolocation_hash: [u8; 32],  // Coordenadas cifradas
        pub is_active: bool,
    }

    #[pallet::storage]
    #[pallet::getter(fn rwa_registry)]
    pub type RwaRegistry<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        [u8; 32], // ID del Token (Derivado del hash de la foto)
        RealWorldAsset,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn rwa_owner)]
    pub type RwaOwner<T: Config> = StorageMap<_, Blake2_128Concat, [u8; 32], T::AccountId, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        AssetTokenized(T::AccountId, [u8; 32]), // Cuenta, Token ID
    }

    #[pallet::error]
    pub enum Error<T> {
        AssetAlreadyExists,
        InvalidFeaSignature,
        IdentityNotVerified,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Emite un token RWA basado en una fotografía y validado por la FEA del usuario.
        #[pallet::call_index(0)]
        #[pallet::weight(10_000 + T::DbWeight::get().writes(2))]
        pub fn tokenize_photo_asset(
            origin: OriginFor<T>,
            photo_hash: [u8; 32],
            fea_signature: [u8; 64],
            geolocation_hash: [u8; 32],
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 1. Validar a través del contrato horizontal de Identidad Soberana (IDS)
            ensure!(
                T::IdentityValidator::is_fea_valid(&who, &fea_signature),
                Error::<T>::InvalidFeaSignature
            );

            ensure!(!RwaRegistry::<T>::contains_key(&photo_hash), Error::<T>::AssetAlreadyExists);

            // 2. Crear el objeto RWA
            let asset = RealWorldAsset {
                photo_hash,
                fea_signature,
                geolocation_hash,
                is_active: true,
            };

            // 3. Registrar y emitir el Token
            RwaRegistry::<T>::insert(&photo_hash, asset);
            RwaOwner::<T>::insert(&photo_hash, &who);

            Self::deposit_event(Event::AssetTokenized(who, photo_hash));

            Ok(())
        }
    }
}