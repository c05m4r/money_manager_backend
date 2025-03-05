from django.db import models
from django.utils.timezone import now
from core.models import BaseModel, SoftDeleteModel
from accounts.exceptions import UserRequiredError
from custom_user.models import CustomUser


class Account(BaseModel, SoftDeleteModel):
    name = models.CharField(max_length=30, unique=True, verbose_name="name")
    users = models.ManyToManyField(
        CustomUser, related_name="accounts", verbose_name="users"
    )

    def __str__(self):
        return self.name
