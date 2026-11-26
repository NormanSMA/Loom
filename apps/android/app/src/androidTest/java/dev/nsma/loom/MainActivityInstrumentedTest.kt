package dev.nsma.loom

import android.Manifest
import android.os.Build
import androidx.test.espresso.Espresso.onView
import androidx.test.espresso.action.ViewActions.click
import androidx.test.espresso.assertion.ViewAssertions.matches
import androidx.test.espresso.matcher.RootMatchers.isDialog
import androidx.test.espresso.matcher.ViewMatchers.isDisplayed
import androidx.test.espresso.matcher.ViewMatchers.withContentDescription
import androidx.test.espresso.matcher.ViewMatchers.withId
import androidx.test.espresso.matcher.ViewMatchers.withText
import androidx.test.ext.junit.rules.ActivityScenarioRule
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.rule.GrantPermissionRule
import org.junit.Rule
import org.junit.Test
import org.junit.rules.RuleChain
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class MainActivityInstrumentedTest {

    private val permissionRule: GrantPermissionRule = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
        GrantPermissionRule.grant(Manifest.permission.POST_NOTIFICATIONS)
    } else {
        GrantPermissionRule.grant()
    }

    private val activityRule = ActivityScenarioRule(MainActivity::class.java)

    @get:Rule
    val ruleChain: RuleChain = RuleChain.outerRule(permissionRule).around(activityRule)

    @Test
    fun topBarControlsWorkAndToggleAccessibility() {
        onView(withId(R.id.statusChip)).check(matches(isDisplayed()))
        onView(withId(R.id.soundButton)).check(matches(isDisplayed()))
        onView(withId(R.id.endpointButton)).check(matches(isDisplayed()))

        onView(withId(R.id.endpointButton)).perform(click())
        onView(withText(R.string.dialog_endpoint_title))
            .inRoot(isDialog())
            .check(matches(isDisplayed()))
        onView(withText(R.string.dialog_cancel))
            .inRoot(isDialog())
            .perform(click())

        onView(withId(R.id.soundButton))
            .check(matches(withContentDescription(R.string.sound_button_mute_desc)))
        onView(withId(R.id.soundButton)).perform(click())
        onView(withId(R.id.soundButton))
            .check(matches(withContentDescription(R.string.sound_button_unmute_desc)))
        onView(withId(R.id.soundButton)).perform(click())
        onView(withId(R.id.soundButton))
            .check(matches(withContentDescription(R.string.sound_button_mute_desc)))
    }
}
