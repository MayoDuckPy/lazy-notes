//@file:OptIn(ExperimentalUnsignedTypes::class)

package com.mayoduckpie.lazy_notes

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import com.mayoduckpie.lazy_notes.ui.theme.LazyNotesTheme

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.windowInsetsTopHeight
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.verticalScroll
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Divider
import androidx.compose.material3.DrawerState
import androidx.compose.material3.DrawerValue
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalDrawerSheet
import androidx.compose.material3.ModalNavigationDrawer
import androidx.compose.material3.NavigationDrawerItem
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.TextButton
import androidx.compose.material3.rememberDrawerState
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.remember
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Settings
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.text.input.ImeAction
import androidx.core.view.WindowCompat
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import com.mayoduckpie.lazy_notes.shared_types.Event
import kotlinx.coroutines.launch

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
                if (core.view?.is_logged_in == true) {
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

    val drawerState = rememberDrawerState(initialValue = DrawerValue.Open)
    ModalNavigationDrawer(
        gesturesEnabled = true,
        drawerState = drawerState,
        drawerContent = {
            ModalDrawerSheet {
                Text(text = "Note Title", modifier = Modifier.padding(16.dp))
                Divider()
                NavigationDrawerItem(
                    label = { Text(text = "Heading 1") },
                    selected = false,
                    onClick = {}
                )
                NavigationDrawerItem(
                    label = { Text(text = "Heading 2") },
                    selected = false,
                    onClick = {}
                )
                NavigationDrawerItem(
                    label = { Text(text = "Heading 3") },
                    selected = false,
                    onClick = {}
                )
                Spacer(Modifier.weight(1f))
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(20.dp)
                        .height(30.dp)
                ) {
                    Text("Welcome, User")
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
                            .fillMaxHeight())
                    // TODO: Replace text with image
                    Text("Sign Out")
                }
            }
        }) {
        Scaffold(
            modifier = Modifier.fillMaxSize(),
            topBar = { Navbar(drawerState) },
            bottomBar = { Buttons(core) }
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
    val scrollState = rememberScrollState()
    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(scrollState)
    ) {
        Text(
            modifier = Modifier.padding(top = 40.dp, bottom = 10.dp),
            text = (core.view?.note ?: "Note not fetched").toString(),
        )
    }
}

@Composable
fun Buttons(core: Core) {
    val coroutineScope = rememberCoroutineScope()
    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Bottom,
        modifier = Modifier.fillMaxWidth()
    ) {
        Row(
//            verticalAlignment = Alignment.Bottom,
            horizontalArrangement = Arrangement.spacedBy(10.dp),
            modifier = Modifier.padding(bottom = 48.dp)
        ) {
            Button(
                onClick = { coroutineScope.launch { core.update(Event.Clear()) } },
                colors = ButtonDefaults.buttonColors(
                    containerColor = MaterialTheme.colorScheme.error
                )
            ) { Text(text = "Clear", color = Color.White) }
            Button(
                onClick = { coroutineScope.launch { core.update(Event.GetNote()) } },
                colors = ButtonDefaults.buttonColors(
                    containerColor = MaterialTheme.colorScheme.secondary
                )
            ) { Text(text = "Get Note", color = Color.White) }
        }
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
            if (core.view?.is_logged_in == true) { noteRoute() }
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

        TextButton( onClick = { signupRoute() } ) {
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
