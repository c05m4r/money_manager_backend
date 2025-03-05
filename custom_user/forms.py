from django.contrib.auth.forms import AdminUserCreationForm, UserChangeForm
from custom_user.models import CustomUser


class CustomUserCreationForm(AdminUserCreationForm):
    class Meta:
        model = CustomUser
        exclude = ["password"]
        # fields = '__all__'


class CustomUserChangeForm(UserChangeForm):
    class Meta:
        model = CustomUser
        exclude = ["password"]
        # fields = '__all__'
