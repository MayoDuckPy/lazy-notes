package com.mayoduckpie.lazy_notes

import com.mayoduckpie.lazy_notes.shared_types.HttpHeader
import com.mayoduckpie.lazy_notes.shared_types.HttpRequest
import com.mayoduckpie.lazy_notes.shared_types.HttpResponse
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.request.headers
import io.ktor.client.request.request
import io.ktor.client.request.setBody
import io.ktor.http.HttpMethod
import io.ktor.util.flattenEntries

suspend fun requestHttp(
    client: HttpClient,
    request: HttpRequest,
): HttpResponse {
    val response = client.request(request.url) {
        this.method = HttpMethod(request.method)
        this.headers {
            for (header in request.headers) {
                append(header.name, header.value)
            }
        }
        setBody(request.body.toByteArray().decodeToString())
    }
    val bytes: ByteArray = response.body()
    val headers = response.headers.flattenEntries().map { HttpHeader(it.first, it.second) }
    return HttpResponse(response.status.value.toShort(), headers, bytes.toList())
}
