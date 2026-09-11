package com.meguru.movel

import java.io.InputStream

/** Selected fields from AOSP debuggerd/proto/tombstone.proto.
 * Never render abort messages, memory dumps, log buffers, or open file paths.
 */
internal object NativeCrashTrace {
  fun read(input: InputStream): String {
    val output = java.io.ByteArrayOutputStream()
    val buffer = ByteArray(8192)
    while (true) {
      val count = input.read(buffer)
      if (count < 0) break
      require(output.size() + count <= 8 * 1024 * 1024) { "Trace too large" }
      output.write(buffer, 0, count)
    }
    return decode(output.toByteArray())
  }

  fun decode(bytes: ByteArray): String {
    val root = Message(bytes, 0, bytes.size)
    val tid = root.number(6)
    require(tid != 0L) { "Missing crash thread" }
    val thread = root.messages(16).firstOrNull { it.number(1) == tid }
      ?.messages(2)?.firstOrNull() ?: error("Missing crash thread")
    val signal = root.messages(10).firstOrNull()
    return buildString {
      appendLine("Native trace v2 · tid=$tid · ${thread.text(2)}")
      if (signal != null) {
        appendLine("${signal.text(2)} ${signal.text(4)} fault=0x${signal.number(9).toULong().toString(16)}")
      }
      val frames = thread.messages(4).take(40).toList()
      require(frames.isNotEmpty()) { "Missing backtrace" }
      frames.forEachIndexed { index, frame ->
        // The library basename and build ID are sufficient for symbolication.
        val library = frame.text(6).substringAfterLast('/')
        appendLine("#$index pc ${frame.number(1).toULong().toString(16)} $library")
        appendLine("  ${frame.text(4)}+${frame.number(5)} build=${frame.text(8)}")
      }
    }
  }

  private class Message(val bytes: ByteArray, val start: Int, val end: Int) {
    private data class Field(val id: Int, val number: Long, val start: Int, val end: Int, val wire: Int)

    private fun fields(): Sequence<Field> = sequence {
      var cursor = start
      fun varint(): Long {
        var result = 0L
        for (shift in 0..63 step 7) {
          require(cursor < end) { "Truncated varint" }
          val value = bytes[cursor++].toInt() and 255
          require(shift != 63 || value <= 1) { "Varint overflow" }
          result = result or ((value and 127).toLong() shl shift)
          if (value < 128) return result
        }
        error("Invalid varint")
      }
      while (cursor < end) {
        val tag = varint()
        require(tag > 0 && tag ushr 3 <= 536870911) { "Invalid tag" }
        val wire = (tag and 7).toInt()
        val id = (tag ushr 3).toInt()
        require(id > 0) { "Invalid field" }
        var number = 0L
        val length = when (wire) {
          0 -> { number = varint(); 0 }
          1 -> 8
          2 -> {
            val size = varint()
            require(size >= 0 && size <= end - cursor) { "Invalid length" }
            size.toInt()
          }
          5 -> 4
          else -> error("Unsupported wire type")
        }
        require(length <= end - cursor) { "Truncated field" }
        yield(Field(id, number, cursor, cursor + length, wire))
        cursor += length
      }
    }

    fun number(id: Int): Long = fields().firstOrNull { it.id == id && it.wire == 0 }?.number ?: 0
    fun messages(id: Int): Sequence<Message> = fields().filter { it.id == id && it.wire == 2 }
      .map { Message(bytes, it.start, it.end) }
    fun text(id: Int): String = messages(id).firstOrNull()?.let {
      String(bytes, it.start, minOf(it.end - it.start, 512), Charsets.UTF_8)
        .replace(Regex("[\\p{Cntrl}]"), " ")
    } ?: ""
  }
}
