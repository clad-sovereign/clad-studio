//! Storage migrations for pallet-clad-token.
//!
//! This module provides a framework for safely upgrading storage schemas during
//! runtime upgrades. Each migration is versioned and runs exactly once.
//!
//! # Migration Pattern
//!
//! When you need to migrate storage:
//!
//! 1. **Increment `STORAGE_VERSION`** in `lib.rs` (e.g., from 1 to 2)
//! 2. **Create a new migration module** (e.g., `v2::MigrateToV2`)
//! 3. **Implement the migration logic** using `OnRuntimeUpgrade`
//! 4. **Add tests** to verify the migration works correctly
//! 5. **Wire up in runtime** via `Executive` type's migration tuple
//!
//! # Example: Adding a New Storage Item
//!
//! ```ignore
//! // In lib.rs, change:
//! const STORAGE_VERSION: StorageVersion = StorageVersion::new(2);
//!
//! // Add new storage:
//! #[pallet::storage]
//! pub type NewFeatureEnabled<T> = StorageValue<_, bool, ValueQuery>;
//!
//! // In migrations.rs, add:
//! pub mod v2 {
//!     use super::*;
//!
//!     pub struct MigrateToV2<T>(PhantomData<T>);
//!
//!     impl<T: Config> OnRuntimeUpgrade for MigrateToV2<T> {
//!         fn on_runtime_upgrade() -> Weight {
//!             let current = Pallet::<T>::on_chain_storage_version();
//!             if current < 2 {
//!                 // Initialize new storage with default value
//!                 NewFeatureEnabled::<T>::put(false);
//!                 StorageVersion::new(2).put::<Pallet<T>>();
//!                 log::info!("Migrated pallet-clad-token storage to v2");
//!                 T::DbWeight::get().reads_writes(1, 2)
//!             } else {
//!                 log::info!("pallet-clad-token already at v2+, skipping migration");
//!                 T::DbWeight::get().reads(1)
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! # Example: Modifying Existing Storage Structure
//!
//! ```ignore
//! // If changing Balances from u128 to a struct:
//! pub mod v3 {
//!     use super::*;
//!
//!     // Define old storage format for reading
//!     mod v2 {
//!         use super::*;
//!         pub type Balances<T: Config> =
//!             StorageMap<_, Blake2_128Concat, T::AccountId, u128, ValueQuery>;
//!     }
//!
//!     pub struct MigrateToV3<T>(PhantomData<T>);
//!
//!     impl<T: Config> OnRuntimeUpgrade for MigrateToV3<T> {
//!         fn on_runtime_upgrade() -> Weight {
//!             let current = Pallet::<T>::on_chain_storage_version();
//!             if current < 3 {
//!                 let mut count: u64 = 0;
//!                 // Iterate over old storage and migrate each entry
//!                 for (account, balance) in v2::Balances::<T>::drain() {
//!                     // Convert to new format and insert
//!                     Balances::<T>::insert(account, NewBalanceStruct { amount: balance, locked: 0 });
//!                     count += 1;
//!                 }
//!                 StorageVersion::new(3).put::<Pallet<T>>();
//!                 T::DbWeight::get().reads_writes(count + 1, count + 1)
//!             } else {
//!                 T::DbWeight::get().reads(1)
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! # Example: Removing Deprecated Storage
//!
//! ```ignore
//! pub mod v4 {
//!     use super::*;
//!
//!     pub struct MigrateToV4<T>(PhantomData<T>);
//!
//!     impl<T: Config> OnRuntimeUpgrade for MigrateToV4<T> {
//!         fn on_runtime_upgrade() -> Weight {
//!             let current = Pallet::<T>::on_chain_storage_version();
//!             if current < 4 {
//!                 // Remove deprecated storage by clearing it
//!                 // Use the raw storage key if the storage item no longer exists in code
//!                 let _ = frame_support::storage::unhashed::clear_prefix(
//!                     &frame_support::storage::storage_prefix(b"CladToken", b"DeprecatedStorage"),
//!                     None,
//!                     None,
//!                 );
//!                 StorageVersion::new(4).put::<Pallet<T>>();
//!                 T::DbWeight::get().reads_writes(1, 1)
//!             } else {
//!                 T::DbWeight::get().reads(1)
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! # Wiring Migrations in Runtime
//!
//! In your runtime's `lib.rs`, add migrations to the `Executive` type:
//!
//! ```ignore
//! // For a single migration:
//! pub type Executive = frame_executive::Executive<
//!     Runtime,
//!     Block,
//!     frame_system::ChainContext<Runtime>,
//!     Runtime,
//!     AllPalletsWithSystem,
//!     pallet_clad_token::migrations::v1::MigrateToV1<Runtime>,
//! >;
//!
//! // For multiple migrations (run in order):
//! pub type Executive = frame_executive::Executive<
//!     Runtime,
//!     Block,
//!     frame_system::ChainContext<Runtime>,
//!     Runtime,
//!     AllPalletsWithSystem,
//!     (
//!         pallet_clad_token::migrations::v1::MigrateToV1<Runtime>,
//!         pallet_clad_token::migrations::v2::MigrateToV2<Runtime>,
//!     ),
//! >;
//! ```
//!
//! # Testing Migrations
//!
//! Always test migrations before deploying:
//!
//! 1. **Unit tests**: Use `try-runtime` feature for pre/post checks
//! 2. **Integration tests**: Run against a fork of mainnet state
//! 3. **Weight validation**: Ensure reported weights match actual execution
//!
//! See the `tests` module for migration testing examples.
//!
//! # Important Guidelines
//!
//! - **Never skip versions**: Always migrate sequentially (v1 → v2 → v3)
//! - **Idempotent migrations**: Check version before migrating to handle re-runs
//! - **Accurate weights**: Return correct `Weight` for actual DB operations
//! - **Logging**: Use `log::info!` to track migration progress
//! - **Backup**: Always have a backup/rollback plan before mainnet migrations

use frame_support::{pallet_prelude::*, traits::OnRuntimeUpgrade};
use sp_std::marker::PhantomData;

use crate::{Config, Pallet};

/// Migration to version 1 (initial release).
///
/// This is a no-op migration that serves as a template. Since v1 is the initial
/// storage version, there's nothing to migrate from v0. This module exists to:
///
/// 1. Document the migration pattern for future developers
/// 2. Provide a working example that compiles and can be tested
/// 3. Establish the framework for subsequent migrations
///
/// Future migrations (v2, v3, etc.) should follow this pattern but implement
/// actual storage transformations.
pub mod v1 {
    use super::*;

    /// Migration struct for upgrading storage to version 1.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The runtime configuration type implementing `Config`
    pub struct MigrateToV1<T>(PhantomData<T>);

    impl<T: Config> OnRuntimeUpgrade for MigrateToV1<T> {
        /// Execute the migration.
        ///
        /// This checks the current on-chain storage version and only runs the
        /// migration if needed. The version check ensures idempotency.
        ///
        /// # Returns
        ///
        /// The weight consumed by this migration (1 read for version check).
        fn on_runtime_upgrade() -> Weight {
            let on_chain_version = Pallet::<T>::on_chain_storage_version();

            if on_chain_version < 1 {
                // Version 0 → 1: Initial release, no storage changes needed.
                // Future migrations would perform actual storage transformations here.
                //
                // Example of what a real migration might do:
                // - Initialize new storage items with default values
                // - Transform existing storage to new format
                // - Clean up deprecated storage

                log::info!(
                    target: "pallet-clad-token",
                    "Running migration v0 → v1 (no-op for initial release)"
                );

                // Update the on-chain storage version
                StorageVersion::new(1).put::<Pallet<T>>();

                // Return weight: 1 read (version check) + 1 write (version update)
                T::DbWeight::get().reads_writes(1, 1)
            } else {
                log::info!(
                    target: "pallet-clad-token",
                    "Storage already at v{on_chain_version:?}, skipping v1 migration"
                );

                // Only performed a read to check the version
                T::DbWeight::get().reads(1)
            }
        }

        /// Pre-upgrade check (requires `try-runtime` feature).
        ///
        /// This runs before `on_runtime_upgrade` to validate preconditions.
        /// Returns encoded state that can be passed to `post_upgrade`.
        #[cfg(feature = "try-runtime")]
        fn pre_upgrade() -> Result<sp_std::vec::Vec<u8>, sp_runtime::TryRuntimeError> {
            let on_chain_version = Pallet::<T>::on_chain_storage_version();
            log::info!(
                target: "pallet-clad-token",
                "Pre-upgrade: on-chain storage version is {:?}",
                on_chain_version
            );

            // Encode any state needed for post_upgrade verification
            Ok(on_chain_version.encode())
        }

        /// Post-upgrade check (requires `try-runtime` feature).
        ///
        /// This runs after `on_runtime_upgrade` to verify the migration succeeded.
        /// Receives the encoded state from `pre_upgrade`.
        #[cfg(feature = "try-runtime")]
        fn post_upgrade(state: sp_std::vec::Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
            let pre_version: u16 = Decode::decode(&mut &state[..])
                .map_err(|_| sp_runtime::TryRuntimeError::Other("Failed to decode pre-state"))?;

            let post_version = Pallet::<T>::on_chain_storage_version();

            log::info!(
                target: "pallet-clad-token",
                "Post-upgrade: version changed from {} to {:?}",
                pre_version,
                post_version
            );

            // Verify migration succeeded if it should have run
            if pre_version < 1 {
                frame_support::ensure!(
                    post_version >= 1,
                    sp_runtime::TryRuntimeError::Other("Migration to v1 did not complete")
                );
            }

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{new_test_ext, Test};
    use frame_support::traits::StorageVersion;

    /// Test that migration correctly updates storage version from 0 to 1.
    #[test]
    fn migration_v1_from_v0_works() {
        new_test_ext().execute_with(|| {
            // Simulate a fresh chain with no storage version set (v0)
            StorageVersion::new(0).put::<Pallet<Test>>();
            assert_eq!(Pallet::<Test>::on_chain_storage_version(), 0);

            // Run the migration
            let _weight = v1::MigrateToV1::<Test>::on_runtime_upgrade();

            // Verify version was updated to 1
            assert_eq!(Pallet::<Test>::on_chain_storage_version(), 1);

            // Note: Weight assertions removed because mock runtime uses DbWeight = ()
            // which returns zero. In production, actual weights would be returned.
        });
    }

    /// Test that migration is idempotent (safe to run multiple times).
    #[test]
    fn migration_v1_idempotent() {
        new_test_ext().execute_with(|| {
            // Start at v1 (already migrated)
            StorageVersion::new(1).put::<Pallet<Test>>();

            // Run migration again
            let _weight = v1::MigrateToV1::<Test>::on_runtime_upgrade();

            // Version should still be 1 (not incremented or changed)
            assert_eq!(Pallet::<Test>::on_chain_storage_version(), 1);
        });
    }

    /// Test that migration doesn't run on higher versions.
    #[test]
    fn migration_v1_skipped_on_higher_version() {
        new_test_ext().execute_with(|| {
            // Simulate future version (e.g., after multiple upgrades)
            StorageVersion::new(5).put::<Pallet<Test>>();

            // Run v1 migration
            let _weight = v1::MigrateToV1::<Test>::on_runtime_upgrade();

            // Version should remain at 5 (migration should be skipped)
            assert_eq!(Pallet::<Test>::on_chain_storage_version(), 5);
        });
    }

    /// Test that migration can handle version 0 (unset) gracefully.
    #[test]
    fn migration_handles_unset_version() {
        new_test_ext().execute_with(|| {
            // Fresh storage has no version set, defaults to 0
            // (GenesisConfig sets it to STORAGE_VERSION, but we override)
            StorageVersion::new(0).put::<Pallet<Test>>();

            // Run migration
            v1::MigrateToV1::<Test>::on_runtime_upgrade();

            // Should now be at v1
            assert_eq!(Pallet::<Test>::on_chain_storage_version(), 1);

            // Running again should have no effect
            v1::MigrateToV1::<Test>::on_runtime_upgrade();
            assert_eq!(Pallet::<Test>::on_chain_storage_version(), 1);
        });
    }
}
