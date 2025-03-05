from django.db import models
from django.utils.timezone import now
from core.models import BaseModel, SoftDeleteModel


class Category(BaseModel, SoftDeleteModel):
    name = models.CharField(max_length=30, unique=True, verbose_name="Name")

    def __str__(self):
        return self.name
