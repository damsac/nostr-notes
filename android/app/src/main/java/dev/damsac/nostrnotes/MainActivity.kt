package dev.damsac.nostrnotes

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.*
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dev.damsac.nostrnotes.rust.AppCore
import dev.damsac.nostrnotes.rust.FfiNote
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val dataDir = filesDir.absolutePath
        setContent {
            MaterialTheme {
                NoteListScreen(dataDir = dataDir)
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun NoteListScreen(dataDir: String) {
    var notes by remember { mutableStateOf<List<FfiNote>>(emptyList()) }
    var isLoading by remember { mutableStateOf(true) }
    var error by remember { mutableStateOf<String?>(null) }
    val scope = rememberCoroutineScope()
    val relayUrl = "wss://nostr.damsac.studio"

    val core = remember {
        try {
            AppCore(relayUrl, dataDir)
        } catch (e: Exception) {
            null
        }
    }
    var coreError by remember {
        mutableStateOf<String?>(
            if (core == null) "Failed to connect. Check your network and try again." else null
        )
    }

    fun refresh() {
        scope.launch {
            isLoading = true
            error = null
            try {
                val appCore = core ?: throw Exception(coreError ?: "Not connected")
                val fetched = withContext(Dispatchers.IO) {
                    appCore.fetchGlobalNotes(50u)
                }
                notes = fetched
            } catch (e: Exception) {
                error = "Could not load notes. Pull to refresh."
            }
            isLoading = false
        }
    }

    LaunchedEffect(Unit) { refresh() }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Notes") },
                actions = {
                    if (notes.isNotEmpty()) {
                        Text(
                            text = "${notes.size}",
                            style = MaterialTheme.typography.labelMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            modifier = Modifier.padding(end = 16.dp)
                        )
                    }
                }
            )
        }
    ) { padding ->
        PullToRefreshBox(
            isRefreshing = isLoading,
            onRefresh = { refresh() },
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
        ) {
            if (error != null && notes.isEmpty()) {
                Column(
                    modifier = Modifier.fillMaxSize(),
                    verticalArrangement = Arrangement.Center,
                    horizontalAlignment = Alignment.CenterHorizontally
                ) {
                    Text(
                        error ?: "Something went wrong",
                        color = MaterialTheme.colorScheme.error
                    )
                    Spacer(modifier = Modifier.height(8.dp))
                    Button(onClick = { refresh() }) { Text("Retry") }
                }
            } else {
                LazyColumn(modifier = Modifier.fillMaxSize()) {
                    items(notes, key = { it.id }) { note ->
                        NoteCard(note)
                    }
                }
            }
        }
    }
}

@Composable
fun NoteCard(note: FfiNote) {
    Card(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 12.dp, vertical = 4.dp)
    ) {
        Column(modifier = Modifier.padding(12.dp)) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween
            ) {
                Text(
                    text = note.displayName,
                    style = MaterialTheme.typography.titleSmall
                )
                Text(
                    text = note.relativeTime,
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
            Spacer(modifier = Modifier.height(6.dp))
            Text(
                text = note.content,
                style = MaterialTheme.typography.bodyMedium,
                maxLines = 6,
                overflow = TextOverflow.Ellipsis
            )
        }
    }
}
