#![cfg_attr(not(feature = "std"), no_std)]

use frame_system::offchain::{
    AppCrypto, CreateSignedTransaction, SendSignedTransaction, Signer,
};
use sp_runtime::offchain::{http, Duration};
use sp_core::crypto::KeyTypeId;

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"leor"); // LEgal ORacle

pub mod crypto {
    use super::KEY_TYPE;
    use sp_runtime::app_crypto::{app_crypto, sr25519};
    app_crypto!(sr25519, KEY_TYPE);
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    #[pallet::disable_frame_system_supertrait_check]
    pub trait Config: frame_system::Config + CreateSignedTransaction<Call<Self>> {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
    }

    /// Almacena la última versión del estándar regulatorio extraído de la BCN
    #[pallet::storage]
    #[pallet::getter(fn current_legal_standard)]
    pub type CurrentLegalStandard<T: Config> = StorageValue<_, [u8; 32], ValueQuery>; // Hash de la norma vigente

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        LegalStandardUpdated([u8; 32]), // Hash del nuevo cuerpo legal aplicable
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn offchain_worker(block_number: BlockNumberFor<T>) {
            // El oráculo revisa las leyes de forma periódica (ej. cada 1000 bloques)
            if block_number % 1000u32.into() == 0u32.into() {
                if let Err(e) = Self::fetch_bcn_open_data_and_update() {
                    log::error!("Fallo en la conexión con la Biblioteca del Congreso Nacional: {:?}", e);
                }
            }
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Inyecta la actualización normativa en la DLT para que el Sandbox la use
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn update_legal_framework(origin: OriginFor<T>, new_law_hash: [u8; 32]) -> DispatchResult {
            ensure_signed(origin)?; // En producción, se exige que sea un nodo validador autorizado
            
            CurrentLegalStandard::<T>::put(new_law_hash);
            Self::deposit_event(Event::LegalStandardUpdated(new_law_hash));
            
            // Nota arquitectónica: Al actualizarse esto, el 'pallet-sandbox-certificador' 
            // exige automáticamente este nuevo hash para aprobar cualquier producto.
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Función para consumir la API de la BCN (Datos Abiertos)
        fn fetch_bcn_open_data_and_update() -> Result<(), &'static str> {
            let signer = Signer::<T, T::AuthorityId>::all_accounts();
            if !signer.can_sign() {
                return Err("No hay cuentas configuradas para firmar la transacción del oráculo legal");
            }

            // Conexión a la base de datos abiertos del Congreso / Normativa Técnica
            let request_url = "https://www.leychile.cl/Consulta/api/normativa_tecnologica_vigente";
            let request = http::Request::get(request_url);
            
            let pending = request.deadline(sp_io::offchain::timestamp().add(Duration::from_millis(5000))).send().map_err(|_| "Error HTTP BCN")?;
            let response = pending.wait().map_err(|_| "Timeout BCN")?;

            if response.code != 200 {
                return Err("Respuesta de la API del Gobierno no es 200 OK");
            }

            // Simulación de parseo del XML/JSON de la ley para extraer el hash de la norma
            let new_law_hash: [u8; 32] = [1u8; 32]; // Hash dummy representativo

            signer.send_signed_transaction(|_acct| {
                Call::update_legal_framework { new_law_hash }
            });

            Ok(())
        }
    }
}