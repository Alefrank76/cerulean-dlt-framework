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

    /// Estructura de mensajería pacs.008 (Transferencia de Crédito de Cliente a Institución)
    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
    pub struct Iso20022Message {
        pub message_id: Vec<u8>, // ID único de transacción End-to-End
        pub debtor_account: Vec<u8>, // IBAN o Identificador de origen
        pub creditor_account: Vec<u8>, // IBAN o Identificador de destino
        pub amount: u64,
        pub currency_code: [u8; 3], // Ej: CLP, USD, EUR
    }

    #[pallet::storage]
    #[pallet::getter(fn financial_messages)]
    pub type FinancialMessages<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        Vec<u8>, // Message ID
        Iso20022Message,
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        Pacs008MessageProcessed(Vec<u8>, u64, [u8; 3]), // Msg ID, Monto, Moneda
    }

    #[pallet::error]
    pub enum Error<T> {
        MessageAlreadyProcessed,
        InvalidCurrencyCode,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Procesa y asienta un mensaje ISO 20022 en la DLT Cerulean
        #[pallet::call_index(0)]
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn process_pacs008_transfer(
            origin: OriginFor<T>,
            message_id: Vec<u8>,
            debtor_account: Vec<u8>,
            creditor_account: Vec<u8>,
            amount: u64,
            currency_code: [u8; 3],
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            ensure!(!FinancialMessages::<T>::contains_key(&message_id), Error::<T>::MessageAlreadyProcessed);

            let msg = Iso20022Message {
                message_id: message_id.clone(),
                debtor_account,
                creditor_account,
                amount,
                currency_code,
            };

            FinancialMessages::<T>::insert(&message_id, msg);
            Self::deposit_event(Event::Pacs008MessageProcessed(message_id, amount, currency_code));

            Ok(())
        }
    }
}