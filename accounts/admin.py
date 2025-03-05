from django.contrib import admin
from django.db import models
from accounts.models import Account


class AccountsAdmin(admin.ModelAdmin):
    model = Account
    exclude_fields = []

    list_display = []
    for field in model._meta.get_fields():
        if (
            not isinstance(
                field,
                (models.ManyToManyField, models.ManyToOneRel, models.ManyToManyRel),
            )
            and field.name not in exclude_fields
        ):
            list_display.append(field.name)

    search_fields = []
    for field in model._meta.get_fields():
        if (
            hasattr(field, "get_internal_type")
            and field.get_internal_type() in ["CharField", "TextField"]
            and field.name not in exclude_fields
        ):
            search_fields.append(field.name)

    list_filter = []
    for field in model._meta.get_fields():
        if (
            hasattr(field, "get_internal_type")
            and field.get_internal_type()
            in ["BooleanField", "DateField", "DateTimeField"]
            and field.name not in exclude_fields
        ):
            list_filter.append(field.name)


admin.site.register(Account, AccountsAdmin)
