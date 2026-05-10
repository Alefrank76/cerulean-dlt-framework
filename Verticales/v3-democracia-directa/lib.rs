#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Configuración de la Vertical de Democracia Directa
    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        
        /// Acoplamiento fuerte con el registro ciudadano horizontal para garantizar
        /// que solo los individuos con atributos cognitivos intactos puedan votar.
        type IdentityVerifier: crate::traits::SovereignIdentityVerifier<Self::AccountId>;
    }

    /// Estructura de una Propuesta Legislativa o Plebiscito
    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct Referendum {
        pub document_hash: [u8; 32],      // Hash del proyecto de ley o minuta
        pub start_block: u32,             // Bloque de inicio de la votación
        pub end_block: u32,               // Bloque de cierre (Deadline)
        pub votes_for: u64,
        pub votes_against: u64,
        pub is_binding: bool,             // Define si el resultado obliga la ejecución
    }

    /// Registro de Referendums activos e históricos.
    #[pallet::storage]
    #[pallet::getter(fn referendums)]
    pub type Referendums<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u32, // Referendum ID
        Referendum,
        OptionQuery,
    >;

    /// Prevención de doble voto: Mapea (Referendum ID, Cuenta) -> Votó (bool)
    #[pallet::storage]
    #[pallet::getter(fn has_voted)]
    pub type HasVoted<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat, u32,           // Referendum ID
        Blake2_128Concat, T::AccountId,  // Cuenta del Ciudadano
        bool,
        ValueQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ReferendumCreated(u32, [u8; 32]), // ID, Hash del documento
        VoteCast(u32, T::AccountId, bool), // ID, Cuenta, ¿A favor?
    }

    #[pallet::error]
    pub enum Error<T> {
        IdentityNotVerified,
        AlreadyVoted,
        ReferendumNotActive,
        ReferendumDoesNotExist,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Emite un voto soberano anclado a la identidad verificada.
        /// Este es el núcleo del sistema 1 Ciudadano = 1 Voto.
        #[pallet::call_index(0)]
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(3, 2))]
        pub fn cast_sovereign_vote(
            origin: OriginFor<T>,
            referendum_id: u32,
            support: bool, // true = A favor, false = En contra
            zkp_signature: [u8; 64], // Prueba de conocimiento cero para mantener anonimato en cabina
        ) -> DispatchResult {
            let voter = ensure_signed(origin)?;

            // 1. Verificación del Origen: ¿Es un ciudadano real y habilitado en la Horizontal?
            ensure!(
                T::IdentityVerifier::is_valid_citizen(&voter, &zkp_signature),
                Error::<T>::IdentityNotVerified
            );

            // 2. Comprobar que no haya votado previamente en este plebiscito
            ensure!(!HasVoted::<T>::get(referendum_id, &voter), Error::<T>::AlreadyVoted);

            // 3. Procesar el voto mutando el estado de la propuesta
            Referendums::<T>::try_mutate(referendum_id, |maybe_ref| -> DispatchResult {
                let referendum = maybe_ref.as_mut().ok_or(Error::<T>::ReferendumDoesNotExist)?;
                
                // (Opcional) Aquí se podría agregar la lógica para validar el número de bloque actual
                // ensure!(current_block <= referendum.end_block, Error::<T>::ReferendumNotActive);

                if support {
                    referendum.votes_for = referendum.votes_for.saturating_add(1);
                } else {
                    referendum.votes_against = referendum.votes_against.saturating_add(1);
                }
                Ok(())
            })?;

            // 4. Registrar que el ciudadano ya emitió su sufragio
            HasVoted::<T>::insert(referendum_id, &voter, true);

            Self::deposit_event(Event::VoteCast(referendum_id, voter, support));

            Ok(())
        }
    }
}