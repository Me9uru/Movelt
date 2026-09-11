package com.meguru.movel

import android.os.Bundle
import android.os.Build
import android.app.ActivityManager
import android.app.ApplicationExitInfo
import android.app.AlertDialog
import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.view.WindowManager
import android.webkit.WebView
import androidx.activity.enableEdgeToEdge
import androidx.activity.OnBackPressedCallback

class MainActivity : TauriActivity() {
  companion object {
    private var crashHandlerInstalled = false
  }

  override fun onCreate(savedInstanceState: Bundle?) {
    installCrashHandler()
    enableEdgeToEdge()
    window.setSoftInputMode(WindowManager.LayoutParams.SOFT_INPUT_ADJUST_NOTHING)
    super.onCreate(savedInstanceState)
    showPreviousCrash()
  }

  private fun installCrashHandler() {
    if (crashHandlerInstalled) return
    crashHandlerInstalled = true
    val preferences = applicationContext.getSharedPreferences("crash-diagnostics", Context.MODE_PRIVATE)
    val previousHandler = Thread.getDefaultUncaughtExceptionHandler()
    Thread.setDefaultUncaughtExceptionHandler { thread, error ->
      try {
        // Exception messages can contain URLs or credentials. Keep only types and frames.
        val report = buildString {
          appendLine("Java/Kotlin 未处理异常 · ${System.currentTimeMillis()}")
          var cause: Throwable? = error
          repeat(4) {
            val current = cause ?: return@repeat
            appendLine(current.javaClass.name)
            current.stackTrace.take(24).forEach { appendLine("  at $it") }
            cause = current.cause
          }
        }
        preferences.edit().putString("java-crash", report).commit()
      } catch (_: Throwable) {
        // Diagnostics must never replace Android's normal crash handling.
      } finally {
        previousHandler?.uncaughtException(thread, error)
      }
    }
  }

  private fun showPreviousCrash() {
    Thread({ collectPreviousCrash() }, "movel-crash-report").start()
  }

  private fun collectPreviousCrash() {
    val preferences = getSharedPreferences("crash-diagnostics", Context.MODE_PRIVATE)
    val javaCrash = preferences.getString("java-crash", null)
    val systemReport = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
      runCatching {
        val manager = getSystemService(Context.ACTIVITY_SERVICE) as ActivityManager
        val seen = preferences.getLong("exit-seen", 0)
        val traceSeen = preferences.getLong("native-trace-seen-v2", 0)
        val exits = manager.getHistoricalProcessExitReasons(packageName, 0, 8)
        val newest = exits.maxOfOrNull { it.timestamp } ?: seen
        preferences.edit().putLong("exit-seen", newest).apply()
        exits.filter {
          (it.timestamp > seen || (
            Build.VERSION.SDK_INT >= Build.VERSION_CODES.S &&
              it.reason == ApplicationExitInfo.REASON_CRASH_NATIVE && it.timestamp > traceSeen
          )) && it.reason in setOf(
            ApplicationExitInfo.REASON_CRASH,
            ApplicationExitInfo.REASON_CRASH_NATIVE,
            ApplicationExitInfo.REASON_ANR,
            ApplicationExitInfo.REASON_LOW_MEMORY,
            ApplicationExitInfo.REASON_SIGNALED,
            ApplicationExitInfo.REASON_EXCESSIVE_RESOURCE_USAGE,
          )
        }.joinToString("\n\n") {
          val reason = when (it.reason) {
            ApplicationExitInfo.REASON_CRASH -> "Java/Kotlin 异常"
            ApplicationExitInfo.REASON_CRASH_NATIVE -> "原生代码崩溃"
            ApplicationExitInfo.REASON_ANR -> "应用无响应"
            ApplicationExitInfo.REASON_LOW_MEMORY -> "系统内存不足"
            ApplicationExitInfo.REASON_SIGNALED -> "系统信号终止"
            else -> "资源使用超限"
          }
          val trace = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S &&
            it.reason == ApplicationExitInfo.REASON_CRASH_NATIVE) {
            val summary = runCatching {
              it.traceInputStream?.use(NativeCrashTrace::read)
                ?: "原生调用栈已不可用，请再次复现闪退。"
            }.getOrElse { error -> "原生调用栈读取失败：${error.javaClass.simpleName}" }
            preferences.edit().putLong("native-trace-seen-v2",
              maxOf(preferences.getLong("native-trace-seen-v2", 0), it.timestamp)).apply()
            "\n$summary"
          } else ""
          "${it.processName}\n$reason (${it.reason}), status=${it.status}" +
            "\ntime=${it.timestamp}, PSS=${it.pss} KB, RSS=${it.rss} KB$trace"
        }
      }.getOrDefault("")
    } else ""
    val pending = preferences.getString("pending-report", null)
    if (javaCrash == null && systemReport.isEmpty()) {
      if (pending != null) showCrashReport(pending)
      return
    }
    val webViewVersion = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
      runCatching { WebView.getCurrentWebViewPackage()?.versionName }.getOrNull()
    } else null
    val report = buildString {
      appendLine("Movel ${BuildConfig.VERSION_NAME} (${BuildConfig.VERSION_CODE})")
      appendLine("Android ${Build.VERSION.RELEASE} (API ${Build.VERSION.SDK_INT})")
      appendLine("${Build.MANUFACTURER} ${Build.MODEL} · ${Build.SUPPORTED_ABIS.joinToString()}")
      appendLine("WebView ${webViewVersion ?: "unknown"}")
      if (javaCrash != null) appendLine(javaCrash)
      append(systemReport)
    }
    // Preserve the report until the user explicitly dismisses or copies it.
    val displayedReport = listOfNotNull(pending, report).joinToString("\n\n").takeLast(16000)
    preferences.edit().putString("pending-report", displayedReport).remove("java-crash").apply()
    showCrashReport(displayedReport)
  }

  private fun showCrashReport(report: String) {
    runOnUiThread {
      if (isFinishing || isDestroyed) return@runOnUiThread
    AlertDialog.Builder(this)
      .setTitle("上次运行异常退出")
      .setMessage("请复制以下诊断信息，用于定位闪退原因。\n\n$report")
      .setCancelable(false)
      .setPositiveButton("复制诊断信息") { _, _ ->
        val clipboard = getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
        clipboard.setPrimaryClip(ClipData.newPlainText("Movel 闪退诊断", report))
        getSharedPreferences("crash-diagnostics", Context.MODE_PRIVATE)
          .edit().remove("pending-report").apply()
      }
      .setNegativeButton("关闭") { _, _ ->
        getSharedPreferences("crash-diagnostics", Context.MODE_PRIVATE)
          .edit().remove("pending-report").apply()
      }
      .show()
    }
  }

  override fun onWebViewCreate(webView: WebView) {
    super.onWebViewCreate(webView)
    onBackPressedDispatcher.addCallback(this, object : OnBackPressedCallback(true) {
      override fun handleOnBackPressed() {
        webView.evaluateJavascript(
          """
          (() => {
            const event = new Event("movel:android-back", { cancelable: true });
            window.dispatchEvent(event);
            return event.defaultPrevented;
          })()
          """.trimIndent()
        ) { handled ->
          if (handled != "true") {
            isEnabled = false
            onBackPressedDispatcher.onBackPressed()
            isEnabled = true
          }
        }
      }
    })
  }
}
