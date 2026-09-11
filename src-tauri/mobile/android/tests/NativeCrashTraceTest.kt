package com.meguru.movel

import org.junit.Assert.*
import org.junit.Test

class NativeCrashTraceTest {
  private fun varint(value: Long): ByteArray {
    var remaining = value
    val bytes = mutableListOf<Byte>()
    do {
      val low = (remaining and 127).toInt()
      remaining = remaining ushr 7
      bytes.add((low or if (remaining != 0L) 128 else 0).toByte())
    } while (remaining != 0L)
    return bytes.toByteArray()
  }
  private fun number(id: Int, value: Long) = varint((id * 8).toLong()) + varint(value)
  private fun field(id: Int, data: ByteArray) = varint((id * 8 + 2).toLong()) + varint(data.size.toLong()) + data
  private fun text(id: Int, value: String) = field(id, value.toByteArray())

  @Test fun extractsOnlyCrashingThreadAndSafeFields() {
    val frame = number(1, 0x1234) + text(4, "reader_load") +
      text(6, "/private/install/libmovel_lib.so") + text(8, "abcd")
    val thread = number(1, 42) + text(2, "tokio-runtime") + field(4, frame) + text(5, "SECRET_MEMORY")
    val other = number(1, 7) + text(2, "SECRET_OTHER_THREAD")
    val root = number(6, 42) + field(16, number(1, 7) + field(2, other)) +
      field(16, number(1, 42) + field(2, thread)) + text(14, "SECRET_ABORT") + text(18, "SECRET_LOG")
    val result = NativeCrashTrace.decode(root)
    assertTrue(result.contains("tid=42"))
    assertTrue(result.contains("pc 1234 libmovel_lib.so"))
    assertTrue(result.contains("reader_load+0 build=abcd"))
    assertFalse(result.contains("SECRET"))
    assertFalse(result.contains("/private"))
  }

  @Test fun rejectsTruncatedAndOverflowingProtobuf() {
    for (bytes in listOf(byteArrayOf(0x80.toByte()), byteArrayOf(0x82.toByte(), 1, 127), ByteArray(11) { 0xff.toByte() })) {
      assertTrue(runCatching { NativeCrashTrace.decode(bytes) }.isFailure)
    }
    assertTrue(runCatching { NativeCrashTrace.decode(byteArrayOf()) }.isFailure)
  }

  @Test fun limitsInputSize() {
    val oversized = ByteArray(8 * 1024 * 1024 + 1).inputStream()
    assertTrue(runCatching { oversized.use(NativeCrashTrace::read) }.isFailure)
  }
}
