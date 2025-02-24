use crate::{Address, OckamError};
use history::ProfileChangeHistory;
use ockam_core::lib::HashMap;
use ockam_node::Context;
use ockam_vault_core::{Hasher, KeyIdVault, PublicKey, Secret, SecretVault, Signer, Verifier};
use ockam_vault_sync_core::VaultSync;

mod authentication;
use authentication::Authentication;
mod contact;
pub use contact::*;
mod identifiers;
pub use identifiers::*;
mod key_attributes;
pub use key_attributes::*;
mod change;
pub use change::*;
mod channel;
pub use channel::*;

pub trait ProfileVault: SecretVault + KeyIdVault + Hasher + Signer + Verifier {}

impl<D> ProfileVault for D where D: SecretVault + KeyIdVault + Hasher + Signer + Verifier {}

pub type ProfileEventAttributes = HashMap<String, String>;
/// Contacts Database
pub type ContactsDb = HashMap<ProfileIdentifier, Contact>;

/// Profile is an abstraction responsible for keeping, verifying and modifying
/// user's data (mainly - public keys). It is used to create new keys, rotate and revoke them.
/// Public keys together with metadata will be organised into events chain, corresponding
/// secret keys will be saved into the given Vault implementation. Events chain and corresponding
/// secret keys are what fully determines Profile.
///
///
/// # Examples
///
/// Create a [`Profile`]. Add and rotate keys.
///
/// ```
/// # use ockam_vault::SoftwareVault;
/// # use ockam::{Profile, ProfileChanges, ProfileSecrets, KeyAttributes, Vault};
/// # fn main() -> ockam_core::Result<()> {
/// # let (mut ctx, mut executor) = ockam_node::start_node();
/// # executor.execute(async move {
/// let vault = Vault::create(&ctx)?;
/// let mut profile = Profile::create(&ctx, &vault)?;
///
/// let root_key_attributes = KeyAttributes::new(
///     Profile::PROFILE_UPDATE.to_string(),
/// );
///
/// let _alice_root_secret = profile.get_secret_key(&root_key_attributes)?;
///
/// let truck_key_attributes = KeyAttributes::new(
///     "Truck management".to_string(),
/// );
///
/// profile.create_key(truck_key_attributes.clone(), None)?;
///
/// let _alice_truck_secret = profile.get_secret_key(&truck_key_attributes)?;
///
/// profile.rotate_key(truck_key_attributes.clone(), None)?;
///
/// let _alice_truck_secret = profile.get_secret_key(&truck_key_attributes)?;
///
/// let verified = profile.verify()?;
/// # ctx.stop().await.unwrap();
/// # Ok::<(), ockam_core::Error>(())
/// # }).unwrap();
/// # Ok(())
/// # }
/// ```
///
/// Authentication using [`Profile`]. In following example Bob authenticates Alice.
///
/// ```
/// # use ockam_vault::SoftwareVault;
/// # use ockam::{Profile, ProfileAuth, ProfileContacts, ProfileSecrets, Vault};
/// fn alice_main() -> ockam_core::Result<()> {
///     # let (mut ctx, mut executor) = ockam_node::start_node();
///     # executor.execute(async move {
///     let vault = Vault::create(&ctx)?;
///
///     // Alice generates profile
///     let mut alice = Profile::create(&ctx, &vault)?;
///
///     // Key agreement happens here
///     let key_agreement_hash = [0u8; 32];
///
///     // Send this over the network to Bob
///     let contact_alice = alice.serialize_to_contact()?;
///     let proof_alice = alice.generate_authentication_proof(&key_agreement_hash)?;
///     # ctx.stop().await.unwrap();
///     # Ok::<(), ockam_core::Error>(())
///     # }).unwrap();
///     Ok(())
/// }
///
/// fn bob_main() -> ockam_core::Result<()> {
///     # let (mut ctx, mut executor) = ockam_node::start_node();
///     # executor.execute(async move {
///     let vault = Vault::create(&ctx)?;
///
///     // Bob generates profile
///     let mut bob = Profile::create(&ctx, &vault)?;
///
///     // Key agreement happens here
///     let key_agreement_hash = [0u8; 32];
///
///     // Receive this from Alice over the network
///     # let contact_alice = [0u8; 32];
///     let contact_alice = Profile::deserialize_contact(&contact_alice)?;
///     let alice_id = contact_alice.identifier().clone();
///
///     // Bob adds Alice to contact list
///     bob.verify_and_add_contact(contact_alice)?;
///
///     # let proof_alice = [0u8; 32];
///     // Bob verifies Alice
///     let verified = bob.verify_authentication_proof(&key_agreement_hash, &alice_id, &proof_alice)?;
///     # ctx.stop().await.unwrap();
///     # Ok::<(), ockam_core::Error>(())
///     # }).unwrap();
///     Ok(())
/// }
/// ```
///
/// Update [`Profile`] and send changes to other parties. In following example Alice rotates
/// her key and sends corresponding [`Profile`] changes to Bob.
///
/// ```
/// # use ockam_vault::SoftwareVault;
/// # use ockam::{Profile, ProfileChanges, ProfileContacts, ProfileSecrets, Vault};
/// fn alice_main() -> ockam_core::Result<()> {
///     # let (mut ctx, mut executor) = ockam_node::start_node();
///     # executor.execute(async move {
///     # let vault = Vault::create(&ctx)?;
///     # let mut alice = Profile::create(&ctx, &vault)?;
///     # let key_agreement_hash = [0u8; 32];
///     # let contact_alice = alice.serialize_to_contact()?;
///     #
///     let index_a = alice.change_events().len();
///     alice.rotate_key(Profile::PROFILE_UPDATE.into(), None)?;
///
///     // Send to Bob
///     let change_events = &alice.change_events()[index_a..];
///     let change_events = Profile::serialize_change_events(change_events)?;
///     # ctx.stop().await.unwrap();
///     # Ok::<(), ockam_core::Error>(())
///     # }).unwrap();
///     Ok(())
/// }
///
/// fn bob_main() -> ockam_core::Result<()> {
///     # let (mut ctx, mut executor) = ockam_node::start_node();
///     # executor.execute(async move {
///     # let vault = Vault::create(&ctx)?;
///     # let mut bob = Profile::create(&ctx, &vault)?;
///     # let key_agreement_hash = [0u8; 32];
///     # let contact_alice = [0u8; 32];
///     # let contact_alice = Profile::deserialize_contact(&contact_alice)?;
///     # let alice_id = contact_alice.identifier().clone();
///     # bob.verify_and_add_contact(contact_alice)?;
///     // Receive from Alice
///     # let change_events = [0u8; 32];
///     let change_events = Profile::deserialize_change_events(&change_events)?;
///     bob.verify_and_update_contact(&alice_id, change_events)?;
///     # ctx.stop().await.unwrap();
///     # Ok::<(), ockam_core::Error>(())
///     # }).unwrap();
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct Profile {
    identifier: ProfileIdentifier,
    change_history: ProfileChangeHistory,
    contacts: ContactsDb,
    vault: VaultSync,
}

impl Profile {
    /// Sha256 of that value is used as previous event id for first event in a [`Profile`]
    pub const NO_EVENT: &'static [u8] = "OCKAM_NO_EVENT".as_bytes();
    /// Label for [`Profile`] update key
    pub const PROFILE_UPDATE: &'static str = "OCKAM_PUK";
    /// Label for key used to issue credentials
    pub const CREDENTIALS_ISSUE: &'static str = "OCKAM_CIK";
    /// Current version of change structure
    pub const CURRENT_CHANGE_VERSION: u8 = 1;
}

pub trait ProfileIdentity {
    /// Return unique [`Profile`] identifier, which is equal to sha256 of the root public key
    fn identifier(&self) -> &ProfileIdentifier;
}

pub trait ProfileChanges {
    /// Return change history chain
    fn change_events(&self) -> &[ProfileChangeEvent];
    /// Add a change event.
    fn update_no_verification(
        &mut self,
        change_event: ProfileChangeEvent,
    ) -> ockam_core::Result<()>;

    /// Verify the whole change event chain
    fn verify(&mut self) -> ockam_core::Result<()>;
}

pub trait ProfileContacts {
    /// Return all known to this profile [`Contact`]s
    fn contacts(&self) -> &ContactsDb;
    /// Convert [`Profile`] to [`Contact`]
    fn to_contact(&self) -> Contact;
    /// Serialize [`Profile`] to [`Contact`] in binary form for storing/transferring over the network
    fn serialize_to_contact(&self) -> ockam_core::Result<Vec<u8>>;
    /// Return [`Contact`] with given [`ProfileIdentifier`]
    fn get_contact(&self, id: &ProfileIdentifier) -> Option<&Contact>;
    /// Verify cryptographically whole event chain. Also verify sequence correctness
    fn verify_contact(&mut self, contact: &Contact) -> ockam_core::Result<()>;
    /// Verify and add new [`Contact`] to [`Profile`]'s Contact list
    fn verify_and_add_contact(&mut self, contact: Contact) -> ockam_core::Result<()>;
    /// Verify and update known [`Contact`] with new [`ProfileChangeEvent`]s
    fn verify_and_update_contact(
        &mut self,
        profile_id: &ProfileIdentifier,
        change_events: Vec<ProfileChangeEvent>,
    ) -> ockam_core::Result<()>;
}

pub trait ProfileAuth {
    fn generate_authentication_proof(
        &mut self,
        channel_state: &[u8],
    ) -> ockam_core::Result<Vec<u8>>;
    fn verify_authentication_proof(
        &mut self,
        channel_state: &[u8],
        responder_contact_id: &ProfileIdentifier,
        proof: &[u8],
    ) -> ockam_core::Result<bool>;
}

/// Supertrait of a Profile
pub trait ProfileTrait: ProfileIdentity + ProfileChanges + ProfileContacts + ProfileAuth {}

impl ProfileIdentity for Profile {
    fn identifier(&self) -> &ProfileIdentifier {
        &self.identifier
    }
}

impl ProfileChanges for Profile {
    fn change_events(&self) -> &[ProfileChangeEvent] {
        self.change_history.as_ref()
    }
    fn update_no_verification(
        &mut self,
        change_event: ProfileChangeEvent,
    ) -> ockam_core::Result<()> {
        let slice = std::slice::from_ref(&change_event);
        ProfileChangeHistory::check_consistency(self.change_events(), &slice)?;
        self.change_history.push_event(change_event);

        Ok(())
    }
    /// Verify whole event chain of current [`Profile`]
    fn verify(&mut self) -> ockam_core::Result<()> {
        ProfileChangeHistory::check_consistency(&[], self.change_events())?;

        self.change_history
            .verify_all_existing_events(&mut self.vault)?;

        let root_public_key = self.change_history.get_first_root_public_key()?;

        let root_key_id = self.vault.compute_key_id_for_public_key(&root_public_key)?;
        let profile_id = ProfileIdentifier::from_key_id(root_key_id);

        if &profile_id != self.identifier() {
            return Err(OckamError::ProfileIdDoesntMatch.into());
        }

        Ok(())
    }
}

impl Profile {
    pub fn new(
        identifier: ProfileIdentifier,
        change_events: Vec<ProfileChangeEvent>,
        contacts: ContactsDb,
        vault: VaultSync,
    ) -> Self {
        let profile = Self {
            identifier,
            change_history: ProfileChangeHistory::new(change_events),
            contacts,
            vault,
        };

        profile
    }
}

pub trait ProfileSecrets {
    /// Create new key. Key is uniquely identified by label in [`KeyAttributes`]
    fn create_key(
        &mut self,
        key_attributes: KeyAttributes,
        attributes: Option<ProfileEventAttributes>,
    ) -> ockam_core::Result<()>;

    /// Rotate existing key. Key is uniquely identified by label in [`KeyAttributes`]
    fn rotate_key(
        &mut self,
        key_attributes: KeyAttributes,
        attributes: Option<ProfileEventAttributes>,
    ) -> ockam_core::Result<()>;

    /// Get [`Secret`] key. Key is uniquely identified by label in [`KeyAttributes`]
    fn get_secret_key(&mut self, key_attributes: &KeyAttributes) -> ockam_core::Result<Secret>;

    /// Get [`PublicKey`]. Key is uniquely identified by label in [`KeyAttributes`]
    fn get_public_key(&self, key_attributes: &KeyAttributes) -> ockam_core::Result<PublicKey>;

    /// Get the root [`Secret`]
    fn get_root_secret(&mut self) -> ockam_core::Result<Secret>;
}

impl ProfileSecrets for Profile {
    fn create_key(
        &mut self,
        key_attributes: KeyAttributes,
        attributes: Option<ProfileEventAttributes>,
    ) -> ockam_core::Result<()> {
        let event = {
            let root_secret = self.get_root_secret()?;
            self.create_key_event(key_attributes, attributes, Some(&root_secret))?
        };
        self.update_no_verification(event)
    }

    fn rotate_key(
        &mut self,
        key_attributes: KeyAttributes,
        attributes: Option<ProfileEventAttributes>,
    ) -> ockam_core::Result<()> {
        let event = {
            let root_secret = self.get_root_secret()?;
            self.rotate_key_event(key_attributes, attributes, &root_secret)?
        };
        self.update_no_verification(event)
    }

    /// Get [`Secret`] key. Key is uniquely identified by label in [`KeyAttributes`]
    fn get_secret_key(&mut self, key_attributes: &KeyAttributes) -> ockam_core::Result<Secret> {
        let event =
            ProfileChangeHistory::find_last_key_event(self.change_events(), key_attributes)?
                .clone();
        Self::get_secret_key_from_event(key_attributes, &event, &mut self.vault)
    }

    fn get_public_key(&self, key_attributes: &KeyAttributes) -> ockam_core::Result<PublicKey> {
        self.change_history.get_public_key(key_attributes)
    }
    fn get_root_secret(&mut self) -> ockam_core::Result<Secret> {
        let public_key =
            ProfileChangeHistory::get_current_profile_update_public_key(self.change_events())?;

        let key_id = self.vault.compute_key_id_for_public_key(&public_key)?;
        self.vault.get_secret_by_key_id(&key_id)
    }
}

impl Profile {
    /// Generate fresh [`Profile`] update key and create new [`Profile`] using it
    fn create_internal(
        attributes: Option<ProfileEventAttributes>,
        mut vault: VaultSync,
    ) -> ockam_core::Result<Self> {
        let prev_id = vault.sha256(Profile::NO_EVENT)?;
        let prev_id = EventIdentifier::from_hash(prev_id);

        let key_attributes = KeyAttributes::new(Profile::PROFILE_UPDATE.to_string());
        let change_event = Self::create_key_event_static(
            prev_id,
            key_attributes.clone(),
            attributes,
            None,
            &mut vault,
        )?;

        let change = ProfileChangeHistory::find_key_change_in_event(&change_event, &key_attributes)
            .ok_or(OckamError::InvalidInternalState)?;
        let public_key = ProfileChangeHistory::get_change_public_key(&change)?;

        let public_kid = vault.compute_key_id_for_public_key(&public_key)?;
        let public_kid = ProfileIdentifier::from_key_id(public_kid);

        let profile = Profile::new(public_kid, vec![change_event], Default::default(), vault);

        Ok(profile)
    }
    /// Generate fresh [`Profile`] update key and create new [`Profile`] using it
    pub fn create_with_attributes(
        attributes: Option<ProfileEventAttributes>,
        ctx: &Context,
        vault: &Address,
    ) -> ockam_core::Result<Self> {
        let vault = VaultSync::create_with_worker(ctx, vault.clone(), "" /* FIXME */)?;

        Self::create_internal(attributes, vault)
    }

    /// Generate fresh [`Profile`] update key and create new [`Profile`] using it
    pub fn create(ctx: &Context, vault: &Address) -> ockam_core::Result<Self> {
        Self::create_with_attributes(None, ctx, vault)
    }
}

impl Profile {
    pub(crate) fn get_secret_key_from_event(
        key_attributes: &KeyAttributes,
        event: &ProfileChangeEvent,
        vault: &mut dyn ProfileVault,
    ) -> ockam_core::Result<Secret> {
        let public_key = ProfileChangeHistory::get_public_key_from_event(key_attributes, event)?;

        let public_kid = vault.compute_key_id_for_public_key(&public_key)?;

        vault.get_secret_by_key_id(&public_kid)
    }
}

impl ProfileContacts for Profile {
    fn contacts(&self) -> &ContactsDb {
        &self.contacts
    }

    fn to_contact(&self) -> Contact {
        Contact::new(
            self.identifier.clone(),
            self.change_history.as_ref().to_vec(),
        )
    }

    fn serialize_to_contact(&self) -> ockam_core::Result<Vec<u8>> {
        let contact = self.to_contact();

        Profile::serialize_contact(&contact)
    }

    fn get_contact(&self, id: &ProfileIdentifier) -> Option<&Contact> {
        self.contacts.get(id)
    }

    fn verify_contact(&mut self, contact: &Contact) -> ockam_core::Result<()> {
        contact.verify(&mut self.vault)
    }

    fn verify_and_add_contact(&mut self, contact: Contact) -> ockam_core::Result<()> {
        self.verify_contact(&contact)?;

        let _ = self.contacts.insert(contact.identifier().clone(), contact);

        Ok(())
    }

    fn verify_and_update_contact(
        &mut self,
        profile_id: &ProfileIdentifier,
        change_events: Vec<ProfileChangeEvent>,
    ) -> ockam_core::Result<()> {
        let contact = self
            .contacts
            .get_mut(profile_id)
            .ok_or(OckamError::ContactNotFound)?;

        contact.verify_and_update(change_events, &mut self.vault)
    }
}

impl Profile {
    /// Serialize [`Contact`] in binary form for storing/transferring over the network
    pub fn serialize_contact(contact: &Contact) -> ockam_core::Result<Vec<u8>> {
        serde_bare::to_vec(&contact).map_err(|_| OckamError::BareError.into())
    }

    /// Deserialize [`Contact`] from binary form
    pub fn deserialize_contact(contact: &[u8]) -> ockam_core::Result<Contact> {
        let contact: Contact =
            serde_bare::from_slice(contact).map_err(|_| OckamError::BareError)?;

        Ok(contact)
    }

    /// Serialize [`ProfileChangeEvent`]s to binary form for storing/transferring over the network
    pub fn serialize_change_events(
        change_events: &[ProfileChangeEvent],
    ) -> ockam_core::Result<Vec<u8>> {
        serde_bare::to_vec(&change_events).map_err(|_| OckamError::BareError.into())
    }

    /// Deserialize [`ProfileChangeEvent`]s from binary form
    pub fn deserialize_change_events(
        change_events: &[u8],
    ) -> ockam_core::Result<Vec<ProfileChangeEvent>> {
        let change_events: Vec<ProfileChangeEvent> =
            serde_bare::from_slice(change_events).map_err(|_| OckamError::BareError)?;

        Ok(change_events)
    }
}

impl ProfileAuth for Profile {
    /// Generate Proof of possession of [`Profile`].
    /// channel_state should be tied to channel's cryptographical material (e.g. h value for Noise XX)
    fn generate_authentication_proof(
        &mut self,
        channel_state: &[u8],
    ) -> ockam_core::Result<Vec<u8>> {
        let root_secret = self.get_root_secret()?;

        Authentication::generate_proof(channel_state, &root_secret, &mut self.vault)
    }

    /// Verify Proof of possession of [`Profile`] with given [`ProfileIdentifier`].
    /// channel_state should be tied to channel's cryptographical material (e.g. h value for Noise XX)
    fn verify_authentication_proof(
        &mut self,
        channel_state: &[u8],
        responder_contact_id: &ProfileIdentifier,
        proof: &[u8],
    ) -> ockam_core::Result<bool> {
        let contact = self
            .get_contact(responder_contact_id)
            .ok_or(OckamError::ContactNotFound)?;

        Authentication::verify_proof(
            channel_state,
            &contact.get_profile_update_public_key()?,
            proof,
            &mut self.vault,
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ockam_vault::SoftwareVault;

    #[test]
    fn test_new() {
        let vault = VaultSync::create_with_mutex(SoftwareVault::default());
        let mut profile = Profile::create_internal(None, vault.start_another().unwrap()).unwrap();

        profile.verify().unwrap();

        let root_key_attributes = KeyAttributes::new(Profile::PROFILE_UPDATE.to_string());

        let _alice_root_secret = profile.get_secret_key(&root_key_attributes).unwrap();
        let _alice_root_public_key = profile.get_public_key(&root_key_attributes).unwrap();

        let truck_key_attributes = KeyAttributes::new("Truck management".to_string());

        profile
            .create_key(truck_key_attributes.clone(), None)
            .unwrap();

        profile.verify().unwrap();

        let _alice_truck_secret = profile.get_secret_key(&truck_key_attributes).unwrap();
        let _alice_truck_public_key = profile.get_public_key(&truck_key_attributes).unwrap();

        profile
            .rotate_key(truck_key_attributes.clone(), None)
            .unwrap();

        profile.verify().unwrap();

        let _alice_truck_secret = profile.get_secret_key(&truck_key_attributes).unwrap();
        let _alice_truck_public_key = profile.get_public_key(&truck_key_attributes).unwrap();

        profile
            .rotate_key(root_key_attributes.clone(), None)
            .unwrap();

        profile.verify().unwrap();

        let _alice_root_secret = profile.get_secret_key(&root_key_attributes).unwrap();
        let _alice_root_public_key = profile.get_public_key(&root_key_attributes).unwrap();
    }

    #[test]
    fn test_update() {
        let vault = VaultSync::create_with_mutex(SoftwareVault::default());
        let mut alice = Profile::create_internal(None, vault.start_another().unwrap()).unwrap();

        let mut bob = Profile::create_internal(None, vault).unwrap();

        // Receive this from Alice over the network
        let contact_alice = alice.serialize_to_contact().unwrap();
        let contact_alice = Profile::deserialize_contact(&contact_alice).unwrap();
        let alice_id = contact_alice.identifier().clone();
        // Bob adds Alice to contact list
        bob.verify_and_add_contact(contact_alice).unwrap();

        alice
            .rotate_key(Profile::PROFILE_UPDATE.into(), None)
            .unwrap();

        let index_a = alice.change_events().len();
        let change_events = &alice.change_events()[index_a..];
        let change_events = Profile::serialize_change_events(change_events).unwrap();

        // Receive from Alice
        let change_events = Profile::deserialize_change_events(&change_events).unwrap();
        bob.verify_and_update_contact(&alice_id, change_events)
            .unwrap();
    }
}
