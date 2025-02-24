use async_trait::async_trait;
use ockam_core::{Address, Result, Routed, Worker};
use ockam_node::Context;
use ockam_vault_core::{
    AsymmetricVault, ErrorVault, Hasher, KeyIdVault, SecretVault, Signer, SymmetricVault, Verifier,
};
use rand::random;
use zeroize::Zeroize;

/// Super-trait of traits required for a Vault Worker.
pub trait VaultTrait:
    AsymmetricVault
    + Hasher
    + KeyIdVault
    + SecretVault
    + Signer
    + SymmetricVault
    + Verifier
    + ErrorVault
    + Send
    + 'static
{
}

impl<V> VaultTrait for V where
    V: AsymmetricVault
        + Hasher
        + KeyIdVault
        + SecretVault
        + Signer
        + SymmetricVault
        + Verifier
        + ErrorVault
        + Send
        + 'static
{
}

mod request_message;
pub(crate) use request_message::*;

mod response_message;
pub(crate) use response_message::*;

/// A Worker that exposes a Vault API.
#[derive(Zeroize)]
pub struct VaultWorker<V>
where
    V: VaultTrait,
{
    inner: V,
}

impl<V> VaultWorker<V>
where
    V: VaultTrait,
{
    /// Create a new VaultWorker.
    fn new(inner: V) -> Self {
        Self { inner }
    }

    /// Start a VaultWorker.
    pub async fn create_with_inner(ctx: &Context, inner: V) -> Result<Address> {
        let address: Address = random();

        ctx.start_worker(address.clone(), Self::new(inner)).await?;

        Ok(address)
    }

    fn handle_request(&mut self, msg: <Self as Worker>::Message) -> Result<VaultResponseMessage> {
        Ok(match msg {
            VaultRequestMessage::EcDiffieHellman {
                context,
                peer_public_key,
            } => {
                let res = self.inner.ec_diffie_hellman(&context, &peer_public_key)?;
                VaultResponseMessage::EcDiffieHellman(res)
            }
            VaultRequestMessage::Sha256 { data } => {
                let res = self.inner.sha256(&data)?;
                VaultResponseMessage::Sha256(res)
            }
            VaultRequestMessage::HkdfSha256 {
                salt,
                info,
                ikm,
                output_attributes,
            } => {
                let res = self
                    .inner
                    .hkdf_sha256(&salt, &info, ikm.as_ref(), output_attributes)?;
                VaultResponseMessage::HkdfSha256(res)
            }
            VaultRequestMessage::GetSecretByKeyId { key_id } => {
                let res = self.inner.get_secret_by_key_id(&key_id)?;
                VaultResponseMessage::GetSecretByKeyId(res)
            }
            VaultRequestMessage::ComputeKeyIdForPublicKey { public_key } => {
                let res = self.inner.compute_key_id_for_public_key(&public_key)?;
                VaultResponseMessage::ComputeKeyIdForPublicKey(res)
            }
            VaultRequestMessage::SecretGenerate { attributes } => {
                let res = self.inner.secret_generate(attributes)?;
                VaultResponseMessage::SecretGenerate(res)
            }
            VaultRequestMessage::SecretImport { secret, attributes } => {
                let res = self.inner.secret_import(&secret, attributes)?;
                VaultResponseMessage::SecretImport(res)
            }
            VaultRequestMessage::SecretExport { context } => {
                let res = self.inner.secret_export(&context)?;
                VaultResponseMessage::SecretExport(res)
            }
            VaultRequestMessage::SecretAttributesGet { context } => {
                let res = self.inner.secret_attributes_get(&context)?;
                VaultResponseMessage::SecretAttributesGet(res)
            }
            VaultRequestMessage::SecretPublicKeyGet { context } => {
                let res = self.inner.secret_public_key_get(&context)?;
                VaultResponseMessage::SecretPublicKeyGet(res)
            }
            VaultRequestMessage::SecretDestroy { context } => {
                self.inner.secret_destroy(context)?;
                VaultResponseMessage::SecretDestroy
            }
            VaultRequestMessage::Sign { secret_key, data } => {
                let res = self.inner.sign(&secret_key, &data)?;
                VaultResponseMessage::Sign(res)
            }
            VaultRequestMessage::AeadAesGcmEncrypt {
                context,
                plaintext,
                nonce,
                aad,
            } => {
                let res = self
                    .inner
                    .aead_aes_gcm_encrypt(&context, &plaintext, &nonce, &aad)?;
                VaultResponseMessage::AeadAesGcmEncrypt(res)
            }
            VaultRequestMessage::AeadAesGcmDecrypt {
                context,
                cipher_text,
                nonce,
                aad,
            } => {
                let res = self
                    .inner
                    .aead_aes_gcm_decrypt(&context, &cipher_text, &nonce, &aad)?;
                VaultResponseMessage::AeadAesGcmDecrypt(res)
            }
            VaultRequestMessage::Verify {
                signature,
                public_key,
                data,
            } => {
                let res = self.inner.verify(&signature, &public_key, &data).is_ok();
                VaultResponseMessage::Verify(res)
            }
        })
    }
}

#[async_trait]
impl<V> Worker for VaultWorker<V>
where
    V: VaultTrait,
{
    type Message = VaultRequestMessage;
    type Context = Context;

    async fn handle_message(
        &mut self,
        ctx: &mut Self::Context,
        msg: Routed<Self::Message>,
    ) -> Result<()> {
        let return_route = msg.return_route();
        let response = self.handle_request(msg.body());

        let response = ResultMessage::new(response);

        ctx.send(return_route, response).await?;

        Ok(())
    }
}
