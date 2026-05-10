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
        type IdentityValidator: crate::traits::SovereignIdentityVerifier<Self::AccountId>;
    }

    /// Clasificación Jurídica del Activo
    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum AssetClass {
        Movable,   // Bien Mueble (Ej: Vehículos, obras de arte, maquinaria)
        Immovable, // Bien Inmueble (Ej: Casas, terrenos - Requiere inscripción CBR)
    }

    /// Estructura de un Activo del Mundo Real (RWA) con exigencia probatoria
    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct RealWorldAsset {
        pub asset_class: AssetClass,
        pub photo_hash: [u8; 32],
        pub fea_signature_pqc: [u8; 64], // Firma Electrónica Avanzada Post-Cuántica
        pub ownership_proof_hash: [u8; 32], // Hash del documento probatorio (Inscripción o Factura)
        pub is_certified: bool, // Determina si pasó por el Sandbox de la Cámara
    }

    #[pallet::storage]
    #[pallet::getter(fn rwa_registry)]
    pub type RwaRegistry<T: Config> = StorageMap<_, Blake2_128Concat, [u8; 32], RealWorldAsset, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        AssetPendingCertification([u8; 32], AssetClass), // El activo entra al Sandbox
        AssetTokenized([u8; 32], T::AccountId), 
    }

    #[pallet::error]
    pub enum Error<T> {
        MissingOwnershipProof,
        InvalidFeaSignature,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Solicita la tokenización inyectando las pruebas de propiedad correspondientes
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn request_tokenization(
            origin: OriginFor<T>,
            asset_class: AssetClass,
            photo_hash: [u8; 32],
            fea_signature_pqc: [u8; 64],
            ownership_proof_hash: [u8; 32], // App exige PDF de CBR o Declaración jurada/factura
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 1. Validar Identidad y FEA Post-Cuántica en la Horizontal
            ensure!(T::IdentityValidator::is_fea_valid(&who, &fea_signature_pqc), Error::<T>::InvalidFeaSignature);

            // 2. Validar que se adjuntó prueba de propiedad
            ensure!(ownership_proof_hash != [0u8; 32], Error::<T>::MissingOwnershipProof);

            let asset = RealWorldAsset {
                asset_class: asset_class.clone(),
                photo_hash,
                fea_signature_pqc,
                ownership_proof_hash,
                is_certified: false, // Entra en estado de cuarentena/sandbox por defecto
            };

            RwaRegistry::<T>::insert(&photo_hash, asset);
            Self::deposit_event(Event::AssetPendingCertification(photo_hash, asset_class));

            Ok(())
        }
    }
}