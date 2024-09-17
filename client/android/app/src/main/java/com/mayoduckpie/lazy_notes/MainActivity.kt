//@file:OptIn(ExperimentalUnsignedTypes::class)

package com.mayoduckpie.lazy_notes

import android.os.Bundle
import android.view.ViewGroup
import android.webkit.WebResourceRequest
import android.webkit.WebResourceResponse
import android.webkit.WebView
import android.webkit.WebViewClient
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.windowInsetsTopHeight
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Divider
import androidx.compose.material3.DrawerState
import androidx.compose.material3.DrawerValue
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalDrawerSheet
import androidx.compose.material3.ModalNavigationDrawer
import androidx.compose.material3.NavigationDrawerItem
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.rememberDrawerState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.core.net.toUri
import androidx.core.view.WindowCompat
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import com.mayoduckpie.lazy_notes.shared_types.Event
import com.mayoduckpie.lazy_notes.shared_types.TocHeading
import com.mayoduckpie.lazy_notes.ui.theme.LazyNotesTheme
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlin.jvm.optionals.getOrNull

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        WindowCompat.setDecorFitsSystemWindows(window, false)
        enableEdgeToEdge()

        setContent {
            LazyNotesTheme {
                Surface(
                    modifier = Modifier.fillMaxSize(), color = MaterialTheme.colorScheme.background
                ) {
                    App()
                }
            }
        }
    }
}

@Composable
fun App(core: Core = viewModel()) {
    // Get session if it exists
    LaunchedEffect(Unit) {
        core.update(Event.GetSession())
    }

    // Navigation
    val navController = rememberNavController()
    NavHost(navController, startDestination = "note") {
        composable("login") {
            LoginView(
                core = core,
                signupRoute = { navController.navigate("signup") },
                noteRoute = {
                    navController.navigate("note")
                    navController.clearBackStack("login")
                }
            )
        }
        composable("note") {
            if (core.view != null) {
                if (core.view?.session!!.isPresent) {
                    NoteView(core)
                } else {
                    navController.navigate("login")
                }
            }
        }
        // composable("signup") { SignupView() }
        // composable("settings") { SettingsView() }
    }
}

@Composable
fun NoteView(core: Core) {
//    val scope = rememberCoroutineScope()
    val drawerState = rememberDrawerState(initialValue = DrawerValue.Closed)

    val toc = core.view?.toc?.getOrNull()
    var tocList: List<TocHeading> = emptyList()
    var title = ""

    if (!toc.isNullOrEmpty()) {
        title = toc[0].text.orElse("Untitled Note")
        tocList = toc
    }

    ModalNavigationDrawer(
        gesturesEnabled = drawerState.isOpen,
        drawerState = drawerState,
        drawerContent = {
            ModalDrawerSheet {
                Scaffold(
                    topBar = {
                        Column(modifier = Modifier.fillMaxWidth()) {
                            Text(text = title, modifier = Modifier.padding(16.dp))
                            Divider(modifier = Modifier.padding(0.dp, 0.dp, 0.dp, 8.dp))
                        }
                    },
                    bottomBar = {
                        Divider()
                        Row(
                            verticalAlignment = Alignment.CenterVertically,
                            modifier = Modifier
                                .fillMaxWidth()
                                .padding(16.dp)
                        ) {
                            Text(
                                "Welcome, ${
                                    core.view?.session?.map { session -> session.username }
                                        ?.orElse("User")
                                }"
                            )
                            Spacer(Modifier.weight(1f))
                            IconButton(
                                modifier = Modifier.size(20.dp),
                                onClick = { }) {
                                Icon(Icons.Default.Settings, "Settings")
                            }
                            Divider(
                                color = MaterialTheme.colorScheme.secondary,
                                modifier = Modifier
                                    .padding(horizontal = 12.dp)
                                    .width(1.dp)
                                    .height(28.dp)
                            )
                            // TODO: Replace text with icon (maybe)
                            Text("Sign Out", modifier = Modifier.clickable {
                                // TODO: Handle sign out
//                                scope.launch {}
                                println("Sign out attempted")
                            })
                        }
                    }) { innerPadding ->
                    LazyColumn(modifier = Modifier.padding(innerPadding)) {
                        if (tocList.size > 1) {
                            for (heading in tocList.listIterator(1)) {
                                item {
                                    NavigationDrawerItem(
                                        label = { Text(text = heading.text.orElse("Untitled Heading")) },
                                        selected = false,
                                        onClick = {}
                                    )
                                }
                            }
                        }
                    }
                }
            }
        }) {
        Scaffold(
            modifier = Modifier.fillMaxSize(),
            topBar = { Navbar(drawerState) },
        ) { innerPadding ->
            Box(modifier = Modifier.padding(innerPadding)) {
                Note(core)
            }
        }
    }
}

@Composable
fun Navbar(drawerState: DrawerState) {
    val scope = rememberCoroutineScope()
    Column {
        // Status bar color
        Row(
            verticalAlignment = Alignment.Top,
            modifier = Modifier
                .windowInsetsTopHeight(WindowInsets.statusBars)
                .fillMaxWidth()
                .background(MaterialTheme.colorScheme.primary)
        ) {}
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .background(MaterialTheme.colorScheme.primary)
        ) {
            TextButton(
                shape = RoundedCornerShape(8.dp),
                modifier = Modifier
                    .size(60.dp),
                onClick = {
                    scope.launch {
                        drawerState.apply {
                            if (isClosed) open() else close()
                        }
                    }
                }
            ) {
                Text(
                    text = "≡",
                    color = Color.Black,
                    fontSize = 42.sp,
                    modifier = Modifier
                        .align(Alignment.CenterVertically)
                        .offset(y = (-10).dp)
                )
            }
            Spacer(modifier = Modifier.weight(1f))
            // Button(
            //     colors = ButtonDefaults.buttonColors(MaterialTheme.colorScheme.inversePrimary),
            //     modifier = Modifier
            //         .size(60.dp)
            //         .padding(12.dp),
            //     onClick = { /*TODO*/ }) {
            // }
        }
    }
}

@Composable
fun Note(core: Core) {
    val instance = core.view?.session!!.map { it.instance.toUri() }.getOrNull()

    Column(modifier = Modifier.fillMaxSize()) {
        AndroidView(
            factory = {
                WebView(it).apply {
                    layoutParams = ViewGroup.LayoutParams(
                        ViewGroup.LayoutParams.MATCH_PARENT,
                        ViewGroup.LayoutParams.MATCH_PARENT
                    )

                    webViewClient = object : WebViewClient() {
                        override fun shouldInterceptRequest(
                            view: WebView,
                            request: WebResourceRequest
                        ): WebResourceResponse? {
                            // Ignore JS loading
                            if (instance == null || request.url.path!!.endsWith(".js") || request.url.path!!.endsWith(
                                    ".wasm"
                                )
                            ) {
                                return WebResourceResponse(null, null, null)
                            }

                            // CSS handler
                            if (request.url.path!!.endsWith(".css")) {
                                // Wait for css to fetch
                                runBlocking {
                                    core.update(Event.GetCss())
                                }

                                // TODO: Properly handle error (display 'Failed to load stylesheet' toast?)
                                val css = core.view?.css!!.orElse("/* Failed to fetch CSS */")
                                return WebResourceResponse(
                                    "text/css",
                                    "utf-8",
                                    css.byteInputStream()
                                )
                            }

                            // Use default handling if not note (will not load since no handler currently)
                            if (request.url.host != instance.host || !request.url.path!!.endsWith(".md")) {
                                return null
                            }

                            // Construct note path
                            var regex = Regex("^/[^/]*/notes/")
                            val path = request.url.path!!.replace(regex, "")

                            // Wait for note to fetch
                            runBlocking {
                                core.update(Event.GetNote(path))
                            }

                            // Strip nav elements
                            regex = Regex("<nav.*</nav>")
                            val html = core.view?.note!!.orElse("<h1>Failed to fetch note</h1>")
                                .replace(regex, "")

                            return WebResourceResponse("text/html", "utf-8", html.byteInputStream())
                        }
                    }

                    settings.javaScriptEnabled = false
                    settings.loadsImagesAutomatically = true
                    settings.loadWithOverviewMode = true
                    settings.useWideViewPort = true
                    settings.setSupportZoom(true)
                    settings.allowFileAccess = false
                    settings.allowContentAccess = false

                    loadUrl("${instance}/index.md")
                }
            },
        )
    }
}

@Composable
fun LoginView(core: Core, noteRoute: () -> Unit, signupRoute: () -> Unit) {
    val coroutineScope = rememberCoroutineScope()

    var instance by remember { mutableStateOf(TextFieldValue("")) }
    var username by remember { mutableStateOf(TextFieldValue("")) }
    var password by remember { mutableStateOf(TextFieldValue("")) }
    val doLogin: () -> Unit = {
        coroutineScope.launch {
            core.update(Event.Login(instance.text, username.text, password.text))
            if (core.view?.session!!.isPresent) { noteRoute() }
        }
    }

    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
        modifier = Modifier
            .padding(55.dp)
            .fillMaxSize()
    ) {
        Text(
            text = "Welcome to\n Lazy Notes",
            style = MaterialTheme.typography.headlineLarge,
            modifier = Modifier.padding(bottom = 60.dp)
        )

        Column(verticalArrangement = Arrangement.spacedBy(15.dp)) {
            OutlinedTextField(
                value = instance,
                onValueChange = { instance = it },
                label = { Text("Instance URL") },
                singleLine = true,
                keyboardOptions = KeyboardOptions(imeAction = ImeAction.Next)
            )

            OutlinedTextField(
                value = username,
                onValueChange = { username = it },
                label = { Text("Username") },
                singleLine = true,
                keyboardOptions = KeyboardOptions(imeAction = ImeAction.Next)
            )

            OutlinedTextField(
                value = password,
                onValueChange = { password = it },
                label = { Text("Password") },
                singleLine = true,
                visualTransformation = PasswordVisualTransformation(),
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password),
                keyboardActions = KeyboardActions(onDone = { doLogin() })
            )
        }

        TextButton(
            modifier = Modifier
                .padding(top = 40.dp)
                .fillMaxWidth(),
            colors = ButtonDefaults.buttonColors(MaterialTheme.colorScheme.primary),
            onClick = doLogin,
        ) {
            Text("Log in to Lazy Notes")
        }

        TextButton(onClick = { signupRoute() }) {
            Text(
                fontSize = 14.sp,
                text = "Register an account"
            )
        }
        // Row(
        //     horizontalArrangement = Arrangement.spacedBy((-8).dp),
        //     verticalAlignment = Alignment.CenterVertically,
        //     modifier = Modifier.offset(x = 8.dp, y = (-4).dp)
        // ) {
        //     Text("Or,", fontSize = 14.sp)
        //     TextButton( onClick = { signupRoute() } ) {
        //         Text(
        //             fontSize = 14.sp,
        //             text = "register an account"
        //         )
        //     }
        // }
    }
}

// @Composable
// fun SignupView() {
//     // TODO: Implement
// }

// @Composable
// fun SettingsView() {
//     // TODO: Implement
// }

@Preview(showBackground = true)
@Composable
fun NoteViewPreview() {
    LazyNotesTheme {
        NoteView(viewModel())
    }
}

@Preview(showBackground = true)
@Composable
fun LoginViewPreview() {
    LazyNotesTheme {
        LoginView(viewModel(), {}, {})
    }
}
