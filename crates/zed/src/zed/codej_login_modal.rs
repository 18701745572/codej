use client::Client;
use editor::Editor;
use gpui::{AppContext, DismissEvent, Entity, EventEmitter, FocusHandle, Focusable, Styled, rems};
use std::sync::Arc;
use ui::{
    Button, Clickable, Color, Context, Headline, HeadlineSize, InteractiveElement, IntoElement,
    Label, LabelSize, ParentElement, Render, StyledExt, StyledTypography, Window, div, h_flex,
    v_flex,
};
use util::ResultExt;
use workspace::ModalView;

pub struct CodeJLoginModal {
    client: Arc<Client>,
    email_editor: Entity<Editor>,
    password_editor: Entity<Editor>,
    focus_handle: FocusHandle,
}

impl EventEmitter<DismissEvent> for CodeJLoginModal {}
impl ModalView for CodeJLoginModal {}

impl Focusable for CodeJLoginModal {
    fn focus_handle(&self, _: &AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl CodeJLoginModal {
    pub fn new(
        client: Arc<Client>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let focus_handle = cx.focus_handle();
        let email_editor = cx.new(|cx| Editor::single_line(window, cx));
        let password_editor = cx.new(|cx| {
            let mut editor = Editor::single_line(window, cx);
            editor.set_masked(true, cx);
            editor
        });
        Self {
            client,
            email_editor,
            password_editor,
            focus_handle,
        }
    }

    fn submit_login(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let email = self
            .email_editor
            .update(cx, |editor, cx| editor.text(cx).to_string());
        let password = self
            .password_editor
            .update(cx, |editor, cx| editor.text(cx).to_string());

        if email.is_empty() || password.is_empty() {
            return;
        }

        let client = self.client.clone();
        cx.spawn(move |cx| async move {
            client
                .sign_in_with_api_credentials(&email, &password, &cx)
                .await
                .log_err();
        })
        .detach_and_log_err(cx);

        cx.emit(DismissEvent);
    }

    fn submit_register(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let email = self
            .email_editor
            .update(cx, |editor, cx| editor.text(cx).to_string());
        let password = self
            .password_editor
            .update(cx, |editor, cx| editor.text(cx).to_string());

        if email.is_empty() || password.is_empty() {
            return;
        }

        let client = self.client.clone();
        cx.spawn(move |cx| async move {
            client
                .sign_in_with_api_register(&email, &password, &cx)
                .await
                .log_err();
        })
        .detach_and_log_err(cx);

        cx.emit(DismissEvent);
    }

    fn use_browser(&mut self, cx: &mut Context<Self>) {
        let client = self.client.clone();
        cx.spawn(move |cx| async move {
            client.sign_in_with_optional_connect(true, &cx).await.log_err();
        })
        .detach_and_log_err(cx);
        cx.emit(DismissEvent);
    }
}

impl Render for CodeJLoginModal {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .key_context("CodeJLoginModal")
            .track_focus(&self.focus_handle(cx))
            .elevation_2(cx)
            .w(rems(28.))
            .p_4()
            .gap_3()
            .bg(cx.theme().colors().editor_background)
            .rounded_lg()
            .border_1()
            .border_color(cx.theme().colors().border_variant)
            .child(
                v_flex()
                    .gap_1()
                    .child(
                        Headline::new("CodeJ 登录".into())
                            .size(HeadlineSize::Small)
                            .color(Color::Default),
                    )
                    .child(
                        Label::new("使用邮箱和密码登录 codej.cn")
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                    ),
            )
            .child(
                v_flex()
                    .gap_2()
                    .child(
                        v_flex()
                            .gap_1()
                            .child(Label::new("邮箱").size(LabelSize::Small))
                            .child(self.email_editor.clone()),
                    )
                    .child(
                        v_flex()
                            .gap_1()
                            .child(Label::new("密码").size(LabelSize::Small))
                            .child(self.password_editor.clone()),
                    ),
            )
            .child(
                v_flex()
                    .gap_2()
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("login", "登录")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.submit_login(window, cx);
                                    })),
                            )
                            .child(
                                Button::new("register", "注册")
                                    .color(Color::Accent)
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.submit_register(window, cx);
                                    })),
                            ),
                    )
                    .child(
                        Button::new("browser", "使用浏览器登录")
                            .color(Color::Muted)
                            .on_click(cx.listener(|this, _, _window, cx| {
                                this.use_browser(cx);
                            })),
                    ),
            )
    }
}
