package com.meguru.movel.credentials

import android.app.Activity
import android.content.Context
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.Base64
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import java.nio.charset.StandardCharsets
import java.security.KeyStore
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

private const val KEYSTORE = "AndroidKeyStore"
private const val KEY_ALIAS = "com.meguru.movel.credentials.v1"
private const val PREFERENCES = "movel.secure_credentials.v1"
private const val GCM_TAG_LENGTH_BITS = 128

@InvokeArg
class AccountArgs { lateinit var account: String }

@InvokeArg
class SetArgs { lateinit var account: String; lateinit var value: String }

@TauriPlugin
class CredentialStorePlugin(private val activity: Activity) : Plugin(activity) {
    private val preferences = activity.getSharedPreferences(PREFERENCES, Context.MODE_PRIVATE)

    @Command
    fun get(invoke: Invoke) {
        try {
            val account = invoke.parseArgs(AccountArgs::class.java).account
            val result = JSObject()
            result.put("value", preferences.getString(account, null)?.let { decrypt(account, it) })
            invoke.resolve(result)
        } catch (error: Exception) {
            invoke.reject("Unable to read secure credential: ${error.message}")
        }
    }

    @Command
    fun set(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(SetArgs::class.java)
            check(preferences.edit().putString(args.account, encrypt(args.account, args.value)).commit()) {
                "SharedPreferences commit failed"
            }
            invoke.resolve()
        } catch (error: Exception) {
            invoke.reject("Unable to save secure credential: ${error.message}")
        }
    }

    @Command
    fun delete(invoke: Invoke) {
        try {
            val account = invoke.parseArgs(AccountArgs::class.java).account
            check(preferences.edit().remove(account).commit()) { "SharedPreferences commit failed" }
            invoke.resolve()
        } catch (error: Exception) {
            invoke.reject("Unable to delete secure credential: ${error.message}")
        }
    }

    private fun encrypt(account: String, value: String): String {
        val cipher = Cipher.getInstance("AES/GCM/NoPadding")
        cipher.init(Cipher.ENCRYPT_MODE, key())
        cipher.updateAAD(account.toByteArray(StandardCharsets.UTF_8))
        return "${encode(cipher.iv)}:${encode(cipher.doFinal(value.toByteArray(StandardCharsets.UTF_8)))}"
    }

    private fun decrypt(account: String, encryptedValue: String): String {
        val parts = encryptedValue.split(":", limit = 2)
        require(parts.size == 2) { "Malformed encrypted credential" }
        val cipher = Cipher.getInstance("AES/GCM/NoPadding")
        cipher.init(Cipher.DECRYPT_MODE, key(), GCMParameterSpec(GCM_TAG_LENGTH_BITS, decode(parts[0])))
        cipher.updateAAD(account.toByteArray(StandardCharsets.UTF_8))
        return String(cipher.doFinal(decode(parts[1])), StandardCharsets.UTF_8)
    }

    private fun key(): SecretKey {
        val keyStore = KeyStore.getInstance(KEYSTORE).apply { load(null) }
        (keyStore.getKey(KEY_ALIAS, null) as? SecretKey)?.let { return it }
        return KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, KEYSTORE).apply {
            init(KeyGenParameterSpec.Builder(KEY_ALIAS, KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT)
                .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
                .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
                .setRandomizedEncryptionRequired(true)
                .build())
        }.generateKey()
    }

    private fun encode(value: ByteArray): String = Base64.encodeToString(value, Base64.NO_WRAP)
    private fun decode(value: String): ByteArray = Base64.decode(value, Base64.NO_WRAP)
}
