package com.mayoduckpie.lazy_notes

import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import androidx.compose.runtime.mutableStateOf
import androidx.datastore.preferences.core.PreferenceDataStoreFactory
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.stringPreferencesKey
import io.ktor.client.HttpClient
import io.ktor.client.engine.cio.CIO
import com.mayoduckpie.lazy_notes.shared.handleResponse
import com.mayoduckpie.lazy_notes.shared.processEvent
import com.mayoduckpie.lazy_notes.shared.view
import com.mayoduckpie.lazy_notes.shared_types.Effect
import com.mayoduckpie.lazy_notes.shared_types.Event
import com.mayoduckpie.lazy_notes.shared_types.KeyValueOperation.Read as KeyValueReadOp
import com.mayoduckpie.lazy_notes.shared_types.HttpResult
import com.mayoduckpie.lazy_notes.shared_types.KeyValueOperation
import com.mayoduckpie.lazy_notes.shared_types.KeyValueOutput
import com.mayoduckpie.lazy_notes.shared_types.Request
import com.mayoduckpie.lazy_notes.shared_types.Requests
import com.mayoduckpie.lazy_notes.shared_types.ViewModel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.map
import okio.Path.Companion.toPath
import java.util.Optional

private const val filesDir = "/data/data/com.mayoduckpie.lazy_notes"
class Core : androidx.lifecycle.ViewModel() {
    var view: ViewModel? by mutableStateOf(null)
        private set

    private val httpClient = HttpClient(CIO)
    private val dataStore = PreferenceDataStoreFactory.createWithPath(produceFile = { "$filesDir/settings.preferences_pb".toPath() })

    suspend fun update(event: Event) {
        val effects = processEvent(event.bincodeSerialize())

        val requests = Requests.bincodeDeserialize(effects)
        for (request in requests) {
            processEffect(request)
        }
    }

    private suspend fun processEffect(request: Request) {
        when (val effect = request.effect) {
            is Effect.Render -> {
                this.view = ViewModel.bincodeDeserialize(view())
            }

            is Effect.Http -> {
                val response = requestHttp(httpClient, effect.value)

                // Respond with HTTP response
                val effects =
                    handleResponse(
                        request.uuid.toByteArray(),
                        HttpResult.Ok(response).bincodeSerialize()
                    )

                // Handle remaining effects
                val requests = Requests.bincodeDeserialize(effects)
                for (req in requests) {
                    processEffect(req)
                }
            }

            is Effect.KeyValue -> {
                val op: KeyValueOperation = effect.value

                if (op is KeyValueReadOp) {
                    val prefKey = stringPreferencesKey(op.value)
                    val dataFlow: Flow<String?> = dataStore.data.map { prefs ->
                        prefs[prefKey]
                    }

                    var data: Optional<List<Byte>> = Optional.ofNullable(dataFlow.firstOrNull()?.toByteArray()?.asList())
                    if (data.isPresent && data.map { it.isEmpty() }.get()) {
                        // Consider empty strings to be null
                        data = Optional.empty()
                    }

                    // Respond with Datastore output
                    val effects = handleResponse(request.uuid.toByteArray(), KeyValueOutput.Read(data).bincodeSerialize())

                    // Handle remaining effects
                    val requests = Requests.bincodeDeserialize(effects)
                    for (req in requests) {
                        processEffect(req)
                    }
                } else {
                    op as KeyValueOperation.Write
                    val prefKey = stringPreferencesKey(op.field0)
                    val value = op.field1.toByteArray().decodeToString()

                    dataStore.edit { prefs ->
                        if (value.isEmpty()) prefs.remove(prefKey) else prefs[prefKey] = value
                    }

                    // Respond with Datastore output
                    /* NOTE: Current write code is infallible or will crash before getting here so
                             assume write is successful. */
                    val effects = handleResponse(request.uuid.toByteArray(), KeyValueOutput.Write(true).bincodeSerialize())

                    // Handle remaining effects
                    val requests = Requests.bincodeDeserialize(effects)
                    for (req in requests) {
                        processEffect(req)
                    }
                }
            }
        }
    }
}
