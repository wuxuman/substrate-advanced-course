#![cfg_attr(not(feature = "std"), no_std)]

pub mod weights;

pub use pallet::*;
pub use weights::WeightInfo;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature="runtime-benchmarks")]
mod benchmarking;


#[frame_support::pallet]
pub mod pallet {

    pub use frame_support::pallet_prelude::*;
	pub use frame_system::pallet_prelude::*;
    use super::*;


    #[pallet::config]
    pub trait Config: frame_system::Config {

        #[pallet::constant]
        type MaxClaimLength:Get<u32>;
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type WeightInfo: WeightInfo;

	}

    #[pallet::pallet]
	pub struct Pallet<T>(_);

    #[pallet::storage]
	#[pallet::getter(fn proofs)]
	pub type Proofs<T:Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8,T::MaxClaimLength>,
        (T::AccountId,T::BlockNumber),
    >;

    #[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ClaimCreated ( T::AccountId , BoundedVec<u8,T::MaxClaimLength>),
        ClaimRevoked(T::AccountId , BoundedVec<u8,T::MaxClaimLength>),
        ClaimTransfered(T::AccountId , BoundedVec<u8,T::MaxClaimLength>,T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T> {
		ProofAlreadyExist,
        ClaimTooLong,
        ClaimNotExist,
        NotClaimOwner,
    }

    #[pallet::hooks]
    impl<T:Config> Hooks<BlockNumberFor<T>> for Pallet<T>{}

    #[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::create_claim(claim.len() as u32))]
		pub fn create_claim(origin: OriginFor<T>, claim: BoundedVec<u8,T::MaxClaimLength>) -> DispatchResultWithPostInfo {
			
			let sender = ensure_signed(origin)?;

            ensure!(!Proofs::<T>::contains_key(&claim),Error::<T>::ProofAlreadyExist);

			Proofs::<T>::insert(
                &claim, 
                (sender.clone(), frame_system::Pallet::<T>::block_number())
            );

			Self::deposit_event(Event::ClaimCreated (sender, claim ));

			Ok(().into())
		}

        #[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::revoke_claim(claim.len() as u32))]
        pub fn revoke_claim(origin: OriginFor<T>, claim: BoundedVec<u8,T::MaxClaimLength>) -> DispatchResultWithPostInfo {
			
			let sender = ensure_signed(origin)?;

            let (owner,_)=Proofs::<T>::get(&claim).ok_or(Error::<T>::ClaimNotExist)?;
            ensure!(owner==sender,Error::<T>::NotClaimOwner);

			Proofs::<T>::remove(&claim);

			Self::deposit_event(Event::ClaimRevoked ( sender, claim ));

			Ok(().into())
		}

        #[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::transfer_claim(claim.len() as u32))]
        pub fn transfer_claim(origin: OriginFor<T>, claim: BoundedVec<u8,T::MaxClaimLength>,receiver:T::AccountId) -> DispatchResultWithPostInfo {
			
			let sender = ensure_signed(origin)?;

            let (owner,_block_number)=Proofs::<T>::get(&claim).ok_or(Error::<T>::ClaimNotExist)?;

            ensure!(owner==sender,Error::<T>::NotClaimOwner);

            //Proofs::<T>::remove(&claim);

            Proofs::<T>::insert(
                &claim, 
                (receiver.clone(), frame_system::Pallet::<T>::block_number())
            );
			
			Self::deposit_event(Event::ClaimTransfered ( sender, claim ,receiver));

			Ok(().into())
		}
    }

}