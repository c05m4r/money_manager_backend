from django.contrib import admin
from django.urls import path
from graphene_django.views import GraphQLView
from core.schema import schema
from debug_toolbar.toolbar import debug_toolbar_urls
from django.views.decorators.csrf import csrf_exempt

urlpatterns = [
    path("admin/", admin.site.urls),
    path("graphql", csrf_exempt(GraphQLView.as_view(graphiql=True, schema=schema))),
] + debug_toolbar_urls()
